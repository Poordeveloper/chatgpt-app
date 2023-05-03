pub mod gpt;
pub mod store;
pub use anyhow;
pub use log;
pub use serde;
pub use serde_json;
pub mod network;
pub use tokio;
pub use once_cell;
pub use uuid;

pub type Result<F, E = anyhow::Error> = anyhow::Result<F, E>;

#[inline]
pub fn get_env(key: &str) -> String {
    std::env::var(key).unwrap_or_default()
}

#[inline]
pub fn get_env_or<T: Into<String>>(key: &str, default_value: T) -> String {
    match std::env::var(key) {
        Ok(v) if !v.is_empty() => v,
        _ => default_value.into(),
    }
}

pub fn init_env() {
    use env_logger::{Env, DEFAULT_FILTER_ENV};
    env_logger::init_from_env(Env::default().filter_or(DEFAULT_FILTER_ENV, "info"));
    if let Ok(v) = ini::Ini::load_from_file(".env") {
        if let Some(section) = v.section(None::<String>) {
            section
                .iter()
                .for_each(|(k, v)| {
                    std::env::set_var(k.to_uppercase(), v);
                    log::info!("{k}={v}");
                });
        }
    }
}

#[derive(serde::Serialize)]
pub struct RespData<T: serde::Serialize> {
    data: T,
    status: &'static str,
}

pub fn resp_data<T: serde::Serialize>(data: T) -> RespData<T> {
    RespData {
        data,
        status: "Success",
    }
}

pub type RespValue = RespData<serde_json::Value>;

#[inline]
pub fn timeout<T: std::future::Future>(ms: u64, future: T) -> tokio::time::Timeout<T> {
    tokio::time::timeout(std::time::Duration::from_millis(ms), future)
}
