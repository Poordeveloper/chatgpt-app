use crate::get_env;

pub fn build_proxy_client() -> Option<reqwest::Client> {
    match build_proxy() {
        Ok(p) => match reqwest::Client::builder().proxy(p).build() {
            Ok(c) => return Some(c),
            Err(err) => log::debug!("Failed to build proxy client: {err}"),
        },
        Err(err) => log::debug!("Faile to build proxy: {err}"),
    }
    None
}

fn build_proxy() -> crate::Result<reqwest::Proxy> {
    use crate::get_env_or;
    let proxy = get_env_or(
        "PROXY",
        get_env_or(
            "HTTPS_PROXY",
            get_env_or(
                "ALL_PROXY",
                get_env_or("HTTP_PROXY", get_env("SOCKS_PROXY")),
            ),
        ),
    );
    if !proxy.is_empty() {
        let url = remove_auth(&proxy);
        log::debug!("Attempt to use proxy: {url}");
        let mut p = reqwest::Proxy::https(url)?;
        if let Some((username, password)) = get_auth(&proxy) {
            log::debug!("Proxy username/password: {username}/{password}");
            p = p.basic_auth(&username, &password);
        }
        return Ok(p);
    }
    anyhow::bail!("No proxy");
}

fn split_schema(url: &str) -> (String, String) {
    let mut ab = url.split("://");
    (
        ab.next().unwrap_or_default().to_string(),
        ab.next().unwrap_or_default().to_string(),
    )
}

fn _remove_auth(url: &str) -> String {
    let mut url = url.to_owned();
    if let Some(pos) = url.find('@') {
        url.replace_range(..pos + 1, "");
    }
    url
}

pub fn remove_auth<T: AsRef<str>>(url: T) -> String {
    let (schema, url) = split_schema(url.as_ref());
    format!("{schema}://{}", _remove_auth(&url))
}

fn get_auth(url: &str) -> Option<(String, String)> {
    if let Some(pos) = url.find('@') {
        let (schema, url) = split_schema(&url);
        let auth = url[..pos - schema.len() - 3].to_string();
        let mut auth = auth.split(':');
        let username = auth.next().unwrap_or_default().to_string();
        let password = auth.next().unwrap_or_default().to_string();
        return Some((username, password));
    }
    None
}

pub async fn fetch(url: &str) -> crate::Result<String> {
    Ok(build_proxy_client()
        .unwrap_or(reqwest::Client::new())
        .get(url)
        .send()
        .await?
        .text()
        .await?)
}

mod test {
    #[test]
    fn test_proxy() {
        assert_eq!(
            super::remove_auth("https://username:password@example.com"),
            "https://example.com"
        );
        assert_eq!(
            super::get_auth("https://username:password@example.com"),
            Some(("username".to_string(), "password".to_string()))
        );
        assert_eq!(
            super::get_auth("socks5://username2:password2@example.com"),
            Some(("username2".to_string(), "password2".to_string()))
        );
        assert_eq!(super::get_auth("https://xample.com"), None);
    }
}
