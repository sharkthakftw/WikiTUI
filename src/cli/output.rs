use std::io::IsTerminal;

pub fn is_stdout_terminal() -> bool {
    std::io::stdout().is_terminal()
}

fn style(s: &str, code: &str) -> String {
    if is_stdout_terminal() {
        format!("\x1b[{}m{}\x1b[0m", code, s)
    } else {
        s.to_string()
    }
}

pub fn bold(s: &str) -> String {
    style(s, "1")
}

pub fn dim(s: &str) -> String {
    style(s, "2")
}

pub fn italic(s: &str) -> String {
    style(s, "3")
}

pub fn cyan(s: &str) -> String {
    style(s, "36")
}

pub fn yellow(s: &str) -> String {
    style(s, "33")
}

pub fn article_url(title: &str) -> String {
    format!("https://en.wikipedia.org/wiki/{}", title.replace(' ', "_"))
}

#[derive(serde::Serialize)]
pub struct ArticleSummaryJson {
    pub title: String,
    pub description: Option<String>,
    pub extract: Option<String>,
    pub url: String,
}

pub fn print_summary(
    title: &str,
    description: Option<&str>,
    extract: Option<&str>,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if json {
        let url = article_url(title);
        let out = ArticleSummaryJson {
            title: title.to_string(),
            description: description.map(|s| s.to_string()),
            extract: extract.map(|s| s.to_string()),
            url,
        };
        println!("{}", serde_json::to_string_pretty(&out)?);
        return Ok(());
    }

    println!("{}", bold(title));
    if let Some(desc) = description
        && !desc.trim().is_empty() {
            println!("{}", italic(&dim(desc.trim())));
        }
    if let Some(ext) = extract
        && !ext.trim().is_empty() {
            println!();
            println!("{}", ext.trim());
        }

    Ok(())
}

pub fn render_line(line: &ratatui::text::Line) -> String {
    if is_stdout_terminal() {
        let mut out = String::new();
        for span in &line.spans {
            let mut prefix = String::new();
            let mut has_style = false;
            if span.style.add_modifier.contains(ratatui::style::Modifier::BOLD) {
                prefix.push_str("\x1b[1m");
                has_style = true;
            }
            if span.style.add_modifier.contains(ratatui::style::Modifier::DIM) {
                prefix.push_str("\x1b[2m");
                has_style = true;
            }
            if span.style.add_modifier.contains(ratatui::style::Modifier::ITALIC) {
                prefix.push_str("\x1b[3m");
                has_style = true;
            }
            if span.style.add_modifier.contains(ratatui::style::Modifier::UNDERLINED) {
                prefix.push_str("\x1b[4m");
                has_style = true;
            }
            if let Some(fg) = span.style.fg {
                match fg {
                    ratatui::style::Color::Rgb(r, g, b) => {
                        prefix.push_str(&format!("\x1b[38;2;{};{};{}m", r, g, b));
                        has_style = true;
                    }
                    ratatui::style::Color::Black => {
                        prefix.push_str("\x1b[30m");
                        has_style = true;
                    }
                    ratatui::style::Color::Red => {
                        prefix.push_str("\x1b[31m");
                        has_style = true;
                    }
                    ratatui::style::Color::Green => {
                        prefix.push_str("\x1b[32m");
                        has_style = true;
                    }
                    ratatui::style::Color::Yellow => {
                        prefix.push_str("\x1b[33m");
                        has_style = true;
                    }
                    ratatui::style::Color::Blue => {
                        prefix.push_str("\x1b[34m");
                        has_style = true;
                    }
                    ratatui::style::Color::Magenta => {
                        prefix.push_str("\x1b[35m");
                        has_style = true;
                    }
                    ratatui::style::Color::Cyan => {
                        prefix.push_str("\x1b[36m");
                        has_style = true;
                    }
                    ratatui::style::Color::White => {
                        prefix.push_str("\x1b[37m");
                        has_style = true;
                    }
                    ratatui::style::Color::Indexed(i) => {
                        prefix.push_str(&format!("\x1b[38;5;{}m", i));
                        has_style = true;
                    }
                    _ => {}
                }
            }
            if let Some(ratatui::style::Color::Rgb(r, g, b)) = span.style.bg {
                prefix.push_str(&format!("\x1b[48;2;{};{};{}m", r, g, b));
                has_style = true;
            }
            if has_style {
                out.push_str(&prefix);
                out.push_str(&span.content);
                out.push_str("\x1b[0m");
            } else {
                out.push_str(&span.content);
            }
        }
        out
    } else {
        line.spans.iter().map(|s| s.content.as_ref()).collect()
    }
}
