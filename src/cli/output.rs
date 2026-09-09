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
