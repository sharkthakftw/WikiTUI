use serde::Deserialize;

#[derive(Deserialize)]
struct SummaryResponse {
    title: Option<String>,
    description: Option<String>,
    extract: Option<String>,
}

fn percent_encode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b'.' || b == b'~' {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{:02X}", b));
        }
    }
    out
}

pub fn fetch_summary(
    agent: &ureq::Agent,
    title: &str,
    timeout_secs: u64,
) -> Result<(String, Option<String>, Option<String>), super::ApiError> {
    let clean_title = title.replace(' ', "_");
    let encoded_title = percent_encode(&clean_title);
    let endpoint = format!(
        "https://en.wikipedia.org/api/rest_v1/page/summary/{}",
        encoded_title
    );
    let resp = agent
        .get(&endpoint)
        .timeout(std::time::Duration::from_secs(timeout_secs.max(1)))
        .call()
        .map_err(|e| super::ApiError::Network(e.to_string()))?;
    let data: SummaryResponse = resp
        .into_json()
        .map_err(|e| super::ApiError::Parse(e.to_string()))?;
    let display_title = data.title.unwrap_or_else(|| title.to_string());
    Ok((display_title, data.description, data.extract))
}
