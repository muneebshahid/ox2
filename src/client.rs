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
    num_retries: Option<u32>,
) -> Result<serde_json::Value, HTTPClientError> {
    let max_retries = num_retries.unwrap_or(3);
    let client = reqwest::Client::new();
    let resolved_headers = headers.unwrap_or_default();
    for retry in 0..=max_retries {
        let request = client
            .post(url)
            .json(payload)
            .headers(resolved_headers.clone());
        let result = request.send().await.and_then(|r| r.error_for_status());
        match result {
            Ok(res) => {
                let parsed = res.json::<serde_json::Value>().await?;
                return Ok(parsed);
            }
            Err(e) => {
                if retry == max_retries {
                    return Err(e.into());
                }
            }
        }
    }
    unreachable!("This point should never be reached");
}
