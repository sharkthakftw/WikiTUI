use std::io::Read;
use std::sync::mpsc::{channel, Receiver};

#[derive(Clone, Copy, Debug)]
pub struct PlayheadUpdate {
    pub is_buffering: bool,
    pub exact_playhead: Option<f64>,
}

pub fn spawn_playhead_monitor<R: Read + Send + 'static>(mut reader: R) -> Receiver<PlayheadUpdate> {
    let (tx, rx) = channel();
    std::thread::spawn(move || {
        let mut buf = Vec::new();
        let mut byte = [0u8; 1];
        while let Ok(1) = reader.read(&mut byte) {
            if byte[0] == b'\r' || byte[0] == b'\n' {
                if !buf.is_empty() {
                    let line = String::from_utf8_lossy(&buf);
                    let trimmed = line.trim();
                    if (trimmed.contains("aq=") || trimmed.contains("fd="))
                        && let Some(first) = trimmed.split_whitespace().next()
                            && let Ok(sec) = first.parse::<f64>() {
                                if sec.is_finite() && sec >= 0.0 {
                                    let _ = tx.send(PlayheadUpdate {
                                        is_buffering: false,
                                        exact_playhead: Some(sec),
                                    });
                                } else {
                                    let _ = tx.send(PlayheadUpdate {
                                        is_buffering: true,
                                        exact_playhead: None,
                                    });
                                }
                            }
                    buf.clear();
                }
            } else {
                buf.push(byte[0]);
            }
        }
    });
    rx
}
