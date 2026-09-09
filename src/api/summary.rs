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

fn fetch_summary_endpoint(
    agent: &ureq::Agent,
    endpoint: &str,
    fallback_title: &str,
    timeout_secs: u64,
) -> Result<(String, Option<String>, Option<String>), super::ApiError> {
    let req = agent.get(endpoint);
    let data: SummaryResponse = super::send_request_json(req, timeout_secs)?;
    let display_title = data.title.unwrap_or_else(|| fallback_title.to_string());
    Ok((display_title, data.description, data.extract))
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
    fetch_summary_endpoint(agent, &endpoint, title, timeout_secs)
}

pub fn fetch_random_summary(
    agent: &ureq::Agent,
    timeout_secs: u64,
) -> Result<(String, Option<String>, Option<String>), super::ApiError> {
    let endpoint = "https://en.wikipedia.org/api/rest_v1/page/random/summary";
    fetch_summary_endpoint(agent, endpoint, "Random Article", timeout_secs)
}
