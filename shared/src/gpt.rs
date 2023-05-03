use crate::{get_env, get_env_or, network::*, resp_data, RespData, Result};
use anyhow::{bail, Context};
use async_openai::{
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestMessageArgs,
        CreateChatCompletionRequest, Role,
    },
    Client, API_BASE,
};
use futures::StreamExt;
use serde_json::{json, Value};
use tiktoken_rs::{async_openai::num_tokens_from_messages, model::get_context_size};

pub const AUTH_SECRET_KEY: &str = "AUTH_SECRET_KEY";
pub const TIMEOUT_ERROR: &str = "OpenAI timed out waiting for response";

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Default)]
pub struct RequestContext {
    #[serde(skip_serializing_if = "Option::is_none", rename = "conversationId")]
    pub conversation_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "parentMessageId")]
    parent_message_id: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct RequestOptions {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub prompt: String,
    #[serde(default, rename = "lastContext")]
    pub last_context: RequestContext,
    #[serde(skip_serializing_if = "Option::is_none", rename = "systemMessage")]
    pub system_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Default)]
pub struct ChatMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    role: Option<Role>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    id: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    text: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    delta: String,
    #[serde(flatten)]
    pub last_context: RequestContext,
}

// https://github.com/transitive-bullshit/chatgpt-api#reverse-proxy
// https://github.com/transitive-bullshit/chatgpt-api/blob/07dcc5df31476fb773a46d103136632e12762179/src/chatgpt-unofficial-proxy-api.ts#L174
// https://github.com/Maxuss/chatgpt_rs/blob/cc2b9a56c937d1d5288d7d2507c02d40c83cadbc/src/client.rs#L67
fn get_client(url: &str, key: &str) -> Client {
    let mut client = Client::new().with_api_key(key);
    if let Some(c) = build_proxy_client() {
        client = client.with_http_client(c);
    }
    if !url.is_empty() {
        client = client.with_api_base(url);
    }
    client
}

fn get_default_client() -> Client {
    get_client(&get_url(), &get_key())
}

fn get_request(opt: RequestOptions, stream: Option<bool>) -> Result<CreateChatCompletionRequest> {
    log::debug!("Request options: {:?}", opt);
    let model = get_model();
    let temperature = opt.temperature;
    let top_p = opt.temperature;
    let (messages, max_tokens, _) = build_messages(&model, opt)?;
    log::debug!("Send messages to OpenAI: {:?}", messages);
    let req = CreateChatCompletionRequest {
        model,
        max_tokens: Some(max_tokens as _),
        messages,
        temperature,
        top_p,
        stream,
        ..Default::default()
    };
    Ok(req)
}

// https://github.com/64bit/async-openai/blob/main/examples/chat-stream/src/main.rs
pub async fn chat_process<F>(
    opt: RequestOptions,
    on_progress: Option<F>,
) -> Result<RespData<ChatMessage>>
where
    F: Fn(ChatMessage),
{
    let stream = if on_progress.is_none() {
        None
    } else {
        Some(true)
    };
    let last_msg = ChatMessage {
        id: uuid::Uuid::new_v4().to_string(),
        last_context: opt.last_context.clone(),
        text: opt.prompt.clone(),
        ..Default::default()
    };

    let mut result = ChatMessage {
        role: Some(Role::Assistant),
        id: uuid::Uuid::new_v4().to_string(),
        last_context: RequestContext {
            conversation_id: opt.last_context.conversation_id.clone(),
            parent_message_id: Some(last_msg.id.clone()),
        },
        ..Default::default()
    };
    let timeout = get_timeout_ms();
    let request = get_request(opt, stream)?;
    match on_progress {
        Some(on_progress) => {
            log::debug!("Start chat stream");
            let mut stream =
                crate::timeout(timeout, get_default_client().chat().create_stream(request))
                    .await
                    .context(TIMEOUT_ERROR)??;
            log::debug!("Start chat stream loop");
            // https://github.com/64bit/async-openai/blob/f6b04b54d5627a18a1f3c376f878290b92ef571a/examples/chat-stream/src/main.rs#L38
            loop {
                let _res = crate::timeout(timeout, stream.next()).await?;
                match _res {
                    Some(Ok(resp)) => {
                        let mut resp = resp;
                        if !resp.choices.is_empty() {
                            let delta = resp.choices.drain(..).nth(0).unwrap().delta;
                            result.delta = delta.content.unwrap_or_default();
                            let text = result.text;
                            result.text = "".to_owned(); // saving bandwidth
                            if delta.role.is_some() {
                                result.role = delta.role;
                            }
                            on_progress(result.clone());
                            result.text = format!("{}{}", text, result.delta);
                        }
                    }
                    Some(Err(err)) => bail!(err),
                    None => break,
                }
            }
        }
        None => {
            // always use the first one: https://github.com/transitive-bullshit/chatgpt-api/blob/bf66500730d0ab4c2388250f3ddac17bf5408df5/src/chatgpt-api.ts#L280
            let mut resp = crate::timeout(timeout, get_default_client().chat().create(request))
                .await
                .context(TIMEOUT_ERROR)??;
            if !resp.choices.is_empty() {
                let msg = resp.choices.drain(..).nth(0).unwrap();
                result.role = Some(msg.message.role);
                result.text = msg.message.content;
            }
        }
    }
    if put_message(&last_msg).is_ok() {
        put_message(&result).ok();
    }
    Ok(resp_data(result))
}

#[inline]
fn get_timeout_ms() -> u64 {
    let i: i32 = get_env("TIMEOUT_MS").parse().unwrap_or(0);
    if i <= 0 {
        30 * 1000
    } else {
        i as _
    }
}

#[inline]
fn get_key() -> String {
    get_env("OPENAI_API_KEY")
}

#[inline]
fn get_model() -> String {
    get_env_or("OPENAI_API_MODEL", "gpt-3.5-turbo")
}

fn get_url() -> String {
    get_env_or("API_REVERSE_PROXY", get_env("OPENAI_API_BASE_URL"))
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Default)]
pub struct DateRange {
    start_date: String,
    end_date: String,
}

pub async fn fetch_usage(rng: DateRange) -> Result<String> {
    let mut url = get_url();
    if url.is_empty() {
        url = API_BASE.to_owned();
    }
    let key = get_key();

    let url_usage = format!(
        "${url}/dashboard/billing/usage?start_date=${}&end_date=${}",
        rng.start_date, rng.end_date
    );
    let client = if let Some(c) = build_proxy_client() {
        c
    } else {
        reqwest::Client::new()
    };
    let res = client
        .get(url_usage)
        .header("Authorization", format!("Bearer ${key}"))
        .header("Content-Type", "application/json")
        .send()
        .await?;
    res.status();
    Ok(res.text().await?)
}

pub async fn chat_config(rng: DateRange, for_web: bool) -> Result<RespData<Value>> {
    let usage = fetch_usage(rng).await.unwrap_or_default();
    let reverse = get_env("API_REVERSE_PROXY");
    let mut proxy = get_env("PROXY");
    if for_web {
        proxy = remove_auth(proxy);
    }
    let mut data = json!({
        "reverseProxy": reverse,
        "proxy": proxy,
        "usage": usage,
        "timeoutMs": get_timeout_ms(),
    });
    if !for_web {
        data["apiKey"] = json!(get_key());
    }
    Ok(resp_data(data))
}

// https://github.com/transitive-bullshit/chatgpt-api/blob/bf66500730d0ab4c2388250f3ddac17bf5408df5/src/chatgpt-api.ts#L361
fn build_messages(
    model: &str,
    opt: RequestOptions,
) -> Result<(Vec<ChatCompletionRequestMessage>, usize, usize)> {
    // https://github.com/transitive-bullshit/chatgpt-api/blob/bf66500730d0ab4c2388250f3ddac17bf5408df5/src/chatgpt-api.ts#L44
    let max_model_tokens = get_context_size(model);
    let max_response_tokens = if model.contains("gpt-4") {
        // if use 32k model
        if model.contains("32k") {
            8192
        } else {
            2048
        }
    } else {
        // https://github.com/transitive-bullshit/chatgpt-api/blob/bf66500730d0ab4c2388250f3ddac17bf5408df5/src/chatgpt-api.ts#L44
        1000
    };
    let max_num_tokens = max_model_tokens - max_response_tokens;
    let mut messages = vec![];
    if let Some(msg) = opt.system_message {
        messages.push(
            ChatCompletionRequestMessageArgs::default()
                .content(msg)
                .role(Role::System)
                .build()?,
        );
    }
    let system_message_offset = messages.len();
    let mut next_messages = if !opt.prompt.is_empty() {
        messages.push(
            ChatCompletionRequestMessageArgs::default()
                .content(opt.prompt)
                .role(Role::User)
                .build()?,
        );
        messages.clone()
    } else {
        messages.clone()
    };
    let mut num_tokens = 0;
    let mut parent_message_id = opt.last_context.parent_message_id;
    loop {
        let next_num_tokens_estimate = num_tokens_from_messages(model, &next_messages)?;
        let is_valid_prompt = next_num_tokens_estimate <= max_num_tokens;

        if !next_messages.is_empty() && !is_valid_prompt {
            break;
        }
        messages = next_messages.clone();
        num_tokens = next_num_tokens_estimate;
        if !is_valid_prompt {
            break;
        }

        if parent_message_id.is_none() {
            break;
        }

        match get_message(&parent_message_id.unwrap()) {
            None => break,
            Some(msg) => {
                let role = msg.role.unwrap_or(Role::User);
                next_messages.insert(
                    system_message_offset,
                    ChatCompletionRequestMessageArgs::default()
                        .content(msg.text)
                        .role(role)
                        .build()?,
                );
                parent_message_id = msg.last_context.parent_message_id;
            }
        }
    }
    let max_tokens = 1.max(max_response_tokens.min(max_model_tokens.saturating_sub(num_tokens)));
    Ok((messages, max_tokens, num_tokens))
}

fn get_message(id: &str) -> Option<ChatMessage> {
    match crate::store::get(id) {
        Ok(Some(data)) => serde_json::from_slice::<ChatMessage>(&data)
            .map(|x| Some(x))
            .unwrap_or(None),
        _ => None,
    }
}

fn put_message(msg: &ChatMessage) -> Result<()> {
    crate::store::put(&msg.id, serde_json::to_vec(&msg)?)?;
    Ok(())
}

pub fn get_session() -> RespData<Value> {
    resp_data(
        json!({"auth": !get_env(AUTH_SECRET_KEY).is_empty(), "isChatGPTAPI": get_env("API_REVERSE_PROXY").is_empty()}),
    )
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Default)]
pub struct TokenBody {
    token: String,
}

pub fn verify(b: TokenBody) -> Result<RespData<Value>> {
    if b.token.is_empty() {
        bail!("Secret key is empty");
    }
    let auth = get_env(AUTH_SECRET_KEY);
    if auth != b.token {
        bail!("Secret key is invalid");
    }
    Ok(resp_data(Value::Null))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize() {
        let msg = ChatMessage::default();
        let res = serde_json::to_string(&msg);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert!(!res.contains("id"));
        assert!(!res.contains("role"));
        let msg = serde_json::from_str::<ChatMessage>(&res);
        assert!(msg.is_ok());
    }
}
