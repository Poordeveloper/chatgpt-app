use tauri::{command, Window};

use shared::{
    gpt::*,
    serde_json::{self, json, Value},
};

// https://tauri.app/v1/guides/features/command/#complete-example

#[command]
pub async fn call(
    window: Window,
    url: String,
    params: Value,
    progress: Option<String>, // originally we use on_progress_id, but it seems that tauri doesn't support this format, always None
) -> std::result::Result<Value, String> {
    shared::log::debug!("{url} {params} {:?}", progress);
    Ok(json!(shared::resp_data({
        _call(window, url, params, progress)
            .await
            .map_err(|x| x.to_string())?
    })))
}

async fn _call(
    window: Window,
    url: String,
    params: Value,
    progress: Option<String>,
) -> shared::Result<Value> {
    let func = if let Some(id) = progress {
        Some(move |data: ChatMessage| {
            let data = serde_json::to_string(&data).unwrap_or_default();
            let val = format!("window['on_progress']['{id}']({{event:{data}}})");
            window.eval(&val).ok();
        })
    } else {
        None
    };
    Ok(match url.as_ref() {
        "/api/session" => json!(get_session()),
        "/api/chat-process" => json!(chat_process(serde_json::from_value(params)?, func).await?),
        "/api/config" => json!(chat_config(serde_json::from_value(params)?, false).await?),
        "/api/verify" | _ => Value::Null,
    })
}

#[command]
pub fn set(key: String, value: String) {
    shared::log::debug!("Set {}={}", key, value);
    std::env::set_var(key, value);
}

#[command]
pub fn get(key: String) -> String {
    std::env::var(key).unwrap_or_default()
}

#[command]
pub async fn fetch(url: String) -> std::result::Result<String, String> {
    Ok(shared::network::fetch(&url)
        .await
        .map_err(|x| x.to_string())?)
}
