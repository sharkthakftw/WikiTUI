use crate::parser::types::{AudioTrack, SpokenAudio};
use crate::parser::utils::decode_html_entities;

pub fn is_spoken_wikipedia_tag(tag: &tl::HTMLTag, _parser: &tl::Parser) -> bool {
    if let Some(id_attr) = tag.attributes().get("id").flatten()
        && id_attr.as_utf8_str() == "spoken-wikipedia" {
            return true;
        }

    if let Some(cls_attr) = tag.attributes().get("class").flatten() {
        let cls = cls_attr.as_utf8_str();
        if cls.contains("spoken-wikipedia") {
            return true;
        }
    }

    false
}

pub fn extract_spoken_audio(tag: &tl::HTMLTag, parser: &tl::Parser) -> Option<SpokenAudio> {
    let binding = tag.inner_text(parser);
    let raw_text = decode_html_entities(&binding);

    let duration = if let Some(open) = raw_text.find('(') {
        if let Some(close) = raw_text[open..].find(')') {
            let inside = &raw_text[open + 1..open + close];
            if inside.contains("minute") || inside.contains("hour") || inside.contains("sec") {
                Some(inside.trim().to_string())
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    let mut tracks = Vec::new();
    collect_audio_sources(tag, parser, &mut tracks);

    if tracks.is_empty() {
        return None;
    }

    // Prefer mp3 streams for maximum player compatibility and lowest stream latency
    tracks.sort_by_key(|t| {
        if t.url.ends_with(".mp3") || t.url.contains(".mp3") {
            0
        } else {
            1
        }
    });

    Some(SpokenAudio {
        title: "Spoken Article".to_string(),
        duration,
        tracks,
    })
}

fn collect_audio_sources<'a>(
    tag: &'a tl::HTMLTag<'a>,
    parser: &'a tl::Parser<'a>,
    tracks: &mut Vec<AudioTrack>,
) {
    let name = tag.name().as_utf8_str();

    if (name == "source" || name == "audio")
        && let Some(src_attr) = tag.attributes().get("src").flatten() {
            let src_bytes = src_attr.as_utf8_str();
            let src_raw = decode_html_entities(&src_bytes);
            let src = if src_raw.starts_with("//") {
                format!("https:{}", src_raw)
            } else {
                src_raw.into_owned()
            };
            if (src.starts_with("http://") || src.starts_with("https://"))
                && !tracks.iter().any(|t| t.url == src)
            {
                let title = if let Some(title_attr) = tag.attributes().get("title").flatten() {
                    decode_html_entities(&title_attr.as_utf8_str()).into_owned()
                } else {
                    format!("Part {}", tracks.len() + 1)
                };
                tracks.push(AudioTrack { title, url: src });
            }
        }

    for child_handle in tag.children().top().iter() {
        if let Some(tl::Node::Tag(child_tag)) = child_handle.get(parser) {
            collect_audio_sources(child_tag, parser, tracks);
        }
    }
}
