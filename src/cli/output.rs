use std::io::IsTerminal;

pub fn is_stdout_terminal() -> bool {
    std::io::stdout().is_terminal()
}

pub fn bold(s: &str) -> String {
    if is_stdout_terminal() {
        format!("\x1b[1m{}\x1b[0m", s)
    } else {
        s.to_string()
    }
}

pub fn dim(s: &str) -> String {
    if is_stdout_terminal() {
        format!("\x1b[2m{}\x1b[0m", s)
    } else {
        s.to_string()
    }
}

pub fn cyan(s: &str) -> String {
    if is_stdout_terminal() {
        format!("\x1b[36m{}\x1b[0m", s)
    } else {
        s.to_string()
    }
}

pub fn yellow(s: &str) -> String {
    if is_stdout_terminal() {
        format!("\x1b[33m{}\x1b[0m", s)
    } else {
        s.to_string()
    }
}
