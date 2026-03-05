const MAX_RETRIES: u32 = 3;
const BASE_DELAY_MS: u64 = 1000;
const MAX_DELAY_MS: u64 = 10000;
const RETRYABLE_ERROR_CODES: [u16; 5] = [429, 500, 502, 503, 504];

#[derive(thiserror::Error, Debug)]
pub enum HTTPClientError {
    #[error("http: {0}")]
    Http(#[from] reqwest::Error),
    #[error("parse: {0}")]
    Parse(#[from] serde_json::Error),
}

fn is_retryable_error(e: &reqwest::Error) -> bool {
    e.is_timeout()
        || e.is_connect()
        || e.status()
            .map_or(false, |code| RETRYABLE_ERROR_CODES.contains(&code.as_u16()))
}

fn backoff_delay(retry: u32) -> std::time::Duration {
    let delay = BASE_DELAY_MS * 2u64.pow(retry) as u64;
    std::time::Duration::from_millis(std::cmp::min(delay, MAX_DELAY_MS))
}

pub async fn post(
    payload: &serde_json::Value,
    headers: Option<reqwest::header::HeaderMap>,
    url: &str,
) -> Result<serde_json::Value, HTTPClientError> {
    let client = reqwest::Client::new();
    let resolved_headers = headers.unwrap_or_default();
    for retry in 0..=MAX_RETRIES {
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
                let is_retryable = is_retryable_error(&e);
                if retry == MAX_RETRIES || !is_retryable {
                    return Err(e.into());
                } else {
                    tokio::time::sleep(backoff_delay(retry)).await;
                }
            }
        }
    }
    unreachable!("This point should never be reached");
}
