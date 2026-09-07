use serde::Deserialize;

#[derive(Deserialize)]
struct ShortenResponse {
    shortenurl: ShortenUrlInner,
}

#[derive(Deserialize)]
struct ShortenUrlInner {
    shorturl: String,
}

pub fn shorten_url(
    agent: &ureq::Agent,
    url: &str,
    timeout_secs: u64,
) -> Result<String, super::ApiError> {
    let endpoint = "https://meta.wikimedia.org/w/api.php";
    let resp = agent
        .post(endpoint)
        .timeout(std::time::Duration::from_secs(timeout_secs.max(1)))
        .send_form(&[("action", "shortenurl"), ("url", url), ("format", "json")])
        .map_err(|e| super::ApiError::Network(e.to_string()))?;
    let data: ShortenResponse = resp
        .into_json()
        .map_err(|e| super::ApiError::Parse(e.to_string()))?;
    Ok(data.shortenurl.shorturl)
}
