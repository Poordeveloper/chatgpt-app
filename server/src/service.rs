use crate::auth::*;
use axum::{
    error_handling::HandleErrorLayer,
    response::IntoResponse,
    routing::{get_service, post},
    Json, Router,
};
use axum_streams::*;
use futures::prelude::*;
use http::{header, Method};
use shared::{anyhow::Context, gpt::ChatMessage, serde::Serialize, *};
use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    result::Result,
};
use tokio::sync::mpsc;
use tower::ServiceBuilder;
use tower_governor::{errors::display_error, governor::GovernorConfigBuilder, GovernorLayer};
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
    trace::TraceLayer,
};

#[tokio::main(flavor = "multi_thread")]
pub async fn run() -> shared::Result<()> {
    let governor_conf = Box::new(
        GovernorConfigBuilder::default()
            .per_second(2)
            .burst_size(5)
            .finish()
            .context("Failed to create rate rate limiter")?,
    );
    let rate_limit_layer = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|e| async move { display_error(e) }))
        .layer(GovernorLayer {
            // We can leak this because it is created once and then
            config: Box::leak(governor_conf),
        });
    let cors = CorsLayer::new()
        //.allow_credentials(true) // conflict with Any
        .allow_headers(vec![
            header::ACCEPT,
            header::ACCEPT_LANGUAGE,
            header::AUTHORIZATION,
            header::CONTENT_LANGUAGE,
            header::CONTENT_TYPE,
            header::USER_AGENT,
        ])
        .allow_methods(vec![
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::HEAD,
            Method::OPTIONS,
            Method::CONNECT,
            Method::PATCH,
            Method::TRACE,
        ])
        .allow_origin(Any);
    let app = Router::new()
        .nest_service("/", get_service(ServeDir::new("asset")))
        .route("/api/session", post(get_session))
        .route(
            "/api/chat-process",
            post(chat_process).route_layer(rate_limit_layer.clone()),
        )
        .route("/api/config", post(config))
        .route("/api/verify", post(verify))
        .layer(TraceLayer::new_for_http())
        .layer(cors);
    let port = get_env_or("PORT", "8080").parse::<u16>()?;
    let addr_v6 = SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), port as _);
    log::info!("Listening on {addr_v6}");
    axum::Server::bind(&addr_v6)
        .serve(
            app.clone()
                .into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .ok();

    log::debug!("Failed to start listening on ipv6, downgrade to ipv4");

    let addr_v4 = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port as _);
    log::info!("Listening on {addr_v4}");
    axum::Server::bind(&addr_v4)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await?;
    Ok(())
}

async fn get_session() -> Result<Json<RespValue>, String> {
    Ok(Json(gpt::get_session()))
}

type Rx = std::sync::Arc<tokio::sync::Mutex<mpsc::UnboundedReceiver<ChatMsgWithRx>>>;
#[derive(Serialize, Default)]
struct ChatMsgWithRx {
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    msg: Option<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<String>,
    #[serde(skip_serializing)]
    rx: Option<Rx>,
}

impl ChatMsgWithRx {
    fn new(rx: Rx) -> Self {
        Self {
            rx: Some(rx),
            ..Default::default()
        }
    }

    fn reset(self, rx: Rx) -> Self {
        let mut x = self;
        x.rx = Some(rx);
        x
    }

    fn new_msg(msg: ChatMessage) -> Self {
        Self {
            msg: Some(msg),
            ..Default::default()
        }
    }

    fn new_err<T: ToString>(err: T) -> Self {
        Self {
            error: Some(err.to_string()),
            ..Default::default()
        }
    }

    fn done() -> Self {
        Self {
            status: Some("Done".to_owned()),
            ..Default::default()
        }
    }
}

async fn chat_process(_: Auth, Json(payload): Json<gpt::RequestOptions>) -> impl IntoResponse {
    let (tx, rx) = mpsc::unbounded_channel();
    tokio::spawn(async move {
        let tx_cloned = tx.clone();
        match gpt::chat_process(
            payload,
            Some(move |msg| {
                tx.send(ChatMsgWithRx::new_msg(msg)).ok();
            }),
        )
        .await
        {
            Err(err) => {
                tx_cloned.send(ChatMsgWithRx::new_err(err)).ok();
            }
            _ => {
                tx_cloned.send(ChatMsgWithRx::done()).ok();
            }
        }
    });

    let rx = std::sync::Arc::new(tokio::sync::Mutex::new(rx));
    let resp_stream = stream::unfold(ChatMsgWithRx::new(rx), |state| async {
        let rx = state.rx.clone().unwrap();
        match state.rx.clone().unwrap().lock().await.recv().await {
            Some(body) => Some((body.reset(rx), state)),
            None => None,
        }
    });
    StreamBodyAs::json_nl(resp_stream)
}

async fn config(_: Auth, Json(payload): Json<gpt::DateRange>) -> Result<Json<RespValue>, String> {
    Ok(Json(
        gpt::chat_config(payload, true).await.map_err(|x| x.to_string())?,
    ))
}

async fn verify(Json(payload): Json<gpt::TokenBody>) -> Result<Json<RespValue>, String> {
    Ok(Json(gpt::verify(payload).map_err(|x| x.to_string())?))
}
