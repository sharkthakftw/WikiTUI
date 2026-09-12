use std::process::{Command, Stdio};

pub fn probe_exact_duration(url: &str) -> Option<u64> {
    let is_http = url.starts_with("http://") || url.starts_with("https://");
    const USER_AGENT: &str = concat!(
        "wikid/",
        env!("CARGO_PKG_VERSION"),
        " (https://github.com/sharkthakftw/wikid)"
    );

    let mut ffprobe_args = vec![
        "-v",
        "error",
        "-show_entries",
        "format=duration",
        "-of",
        "default=noprint_wrappers=1:nokey=1",
    ];
    if is_http {
        ffprobe_args.extend(["-user_agent", USER_AGENT]);
    }
    ffprobe_args.push(url);

    if let Ok(output) = Command::new("ffprobe")
        .args(&ffprobe_args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        && output.status.success() {
            let out_str = String::from_utf8_lossy(&output.stdout);
            if let Ok(f) = out_str.trim().parse::<f64>()
                && f > 0.0 {
                    return Some(f.round() as u64);
                }
        }

    if let Some(file_name) = crate::audio::cache::extract_wikimedia_file_title(url) {
        let api_url = format!(
            "https://en.wikipedia.org/w/api.php?action=query&titles=File:{}&prop=imageinfo&iiprop=duration&format=json",
            file_name
        );
        if let Ok(output) = Command::new("curl")
            .args(["-s", "-A", USER_AGENT, &api_url])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            && output.status.success() {
                let body = String::from_utf8_lossy(&output.stdout);
                if let Some(dur_idx) = body.find("\"duration\":") {
                    let rest = &body[dur_idx + 11..];
                    let end = rest
                        .find(|c: char| !c.is_ascii_digit() && c != '.')
                        .unwrap_or(rest.len());
                    if let Ok(f) = rest[..end].trim().parse::<f64>()
                        && f > 0.0 {
                            return Some(f.round() as u64);
                        }
                }
            }
    }

    None
}
