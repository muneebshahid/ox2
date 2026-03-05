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
    let mut last_response = None;
    for retry in 0..=max_retries {
        let request = client
            .post(url)
            .json(payload)
            .headers(headers.clone().unwrap_or_default());
        let response = request.send().await;
        match response {
            Ok(res) => match res.error_for_status() {
                Ok(res) => {
                    last_response = Some(res);
                    break;
                }
                Err(e) => {
                    if retry == max_retries {
                        return Err(e.into());
                    }
                }
            },
            Err(e) => {
                if retry == max_retries {
                    return Err(e.into());
                }
            }
        }
    }
    let parsed_response = last_response.unwrap().json::<serde_json::Value>().await?;
    Ok(parsed_response)
}
