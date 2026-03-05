#[derive(thiserror::Error, Debug)]
pub enum HTTPClientError {
    #[error("http: {0}")]
    Http(#[from] reqwest::Error),
    #[error("parse: {0}")]
    Parse(#[from] serde_json::Error),
}

pub async fn post(
    payload: &serde_json::Value,
    headers: Option<reqwest::header::HeaderMap>,
    url: &str,
) -> Result<serde_json::Value, HTTPClientError> {
    let client = reqwest::Client::new();
    let request = client
        .post(url)
        .json(payload)
        .headers(headers.unwrap_or_default());
    let response = request
        .send()
        .await?
        .error_for_status()?
        .json::<serde_json::Value>()
        .await?;
    Ok(response)
}
