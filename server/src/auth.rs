use axum::{
    extract::{FromRequestParts, TypedHeader},
    http::StatusCode,
    RequestPartsExt,
};
use headers::{authorization::Bearer, Authorization};
use http::request::Parts;

pub struct Auth(String);

#[async_trait::async_trait]
impl<S> FromRequestParts<S> for Auth
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let key = shared::get_env(shared::gpt::AUTH_SECRET_KEY);
        if key.is_empty() {
            return Ok(Self("".to_owned()));
        }
        // Extract the token from the authorization header
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| StatusCode::UNAUTHORIZED)?;
        if bearer.token() != key {
            return Err(StatusCode::UNAUTHORIZED);
        }
        Ok(Self(bearer.token().to_owned()))
    }
}
