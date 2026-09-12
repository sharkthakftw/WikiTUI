pub fn url_decode(s: &str) -> String {
    let mut bytes = Vec::with_capacity(s.len());
    let mut iter = s.as_bytes().iter().copied();
    while let Some(b) = iter.next() {
        match b {
            b'%' => {
                let hex = iter.next().zip(iter.next()).and_then(|(h1, h2)| {
                    let d1 = (h1 as char).to_digit(16)?;
                    let d2 = (h2 as char).to_digit(16)?;
                    Some((d1 * 16 + d2) as u8)
                });
                if let Some(val) = hex {
                    bytes.push(val);
                } else {
                    bytes.push(b'%');
                }
            }
            b'+' => bytes.push(b' '),
            other => bytes.push(other),
        }
    }
    String::from_utf8(bytes).unwrap_or_else(|e| String::from_utf8_lossy(e.as_bytes()).into_owned())
}

pub(crate) fn extract_title_from_href(href: &str) -> Option<String> {
    let decoded = decode_html_entities(href);
    let trimmed = decoded.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Some(hash_idx) = trimmed.find('#') {
        let anchor = &trimmed[hash_idx..];
        if anchor.starts_with("#cite_note") || anchor.starts_with("#cite_ref") {
            return Some(anchor.to_string());
        }
    }

    if let Some(anchor) = trimmed.strip_prefix('#') {
        if anchor.starts_with("cite_note") || anchor.starts_with("cite_ref") {
            return Some(trimmed.to_string());
        }
        return None;
    }

    const WIKI_PREFIXES: &[&str] = &[
        "/wiki/",
        "./",
        "https://en.wikipedia.org/wiki/",
        "http://en.wikipedia.org/wiki/",
        "//en.wikipedia.org/wiki/",
    ];

    let wiki_path = WIKI_PREFIXES
        .iter()
        .find_map(|p| trimmed.strip_prefix(p))
        .or_else(|| {
            trimmed
                .find("/w/index.php?title=")
                .map(|idx| &trimmed[idx + 19..])
        });

    if let Some(path) = wiki_path {
        let raw_title = path.split('#').next().unwrap_or(path);
        let raw_title = raw_title.split('&').next().unwrap_or(raw_title);

        if let Some(colon_idx) = raw_title.find(':') {
            let prefix = &raw_title[..colon_idx];
            if matches!(
                prefix,
                "Special"
                    | "File"
                    | "Category"
                    | "Help"
                    | "Wikipedia"
                    | "Template"
                    | "User"
                    | "Talk"
                    | "Portal"
                    | "Draft"
                    | "MediaWiki"
                    | "Media"
            ) || prefix.ends_with("_talk")
            {
                return None;
            }
        }
        let decoded = url_decode(raw_title).replace('_', " ");
        let cleaned = decoded.trim().to_string();
        if !cleaned.is_empty() {
            return Some(cleaned);
        }
    }

    if trimmed.starts_with("http://")
        || trimmed.starts_with("https://")
        || trimmed.starts_with("//")
    {
        let full_url = if trimmed.starts_with("//") {
            format!("https:{}", trimmed)
        } else {
            trimmed.to_string()
        };
        return Some(full_url);
    }

    None
}

pub fn extract_domain(url: &str) -> Option<String> {
    let without_proto = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .or_else(|| url.strip_prefix("//"))?;

    let host = without_proto
        .split('/')
        .next()?
        .split('?')
        .next()?
        .split('#')
        .next()?
        .split(':')
        .next()?;

    let host_trimmed = host.trim();
    if host_trimmed.is_empty() {
        return None;
    }

    let clean_host = if let Some(h) = host_trimmed.strip_prefix("www.") {
        h
    } else {
        host_trimmed
    };

    Some(clean_host.to_lowercase())
}

pub fn decode_html_entities(s: &str) -> std::borrow::Cow<'_, str> {
    if !s.contains('&') {
        return std::borrow::Cow::Borrowed(s);
    }

    let mut result = String::with_capacity(s.len());
    let mut chars = s.char_indices().peekable();

    while let Some((i, c)) = chars.next() {
        if c == '&' {
            let remaining = &s[i..];
            if let Some(semicolon_idx) = remaining.find(';')
                && semicolon_idx <= 10 {
                    let entity = &remaining[1..semicolon_idx];
                    let decoded_char = if let Some(num_str) = entity.strip_prefix('#') {
                        if let Some(hex_str) = num_str
                            .strip_prefix('x')
                            .or_else(|| num_str.strip_prefix('X'))
                        {
                            u32::from_str_radix(hex_str, 16)
                                .ok()
                                .and_then(char::from_u32)
                        } else {
                            num_str.parse::<u32>().ok().and_then(char::from_u32)
                        }
                    } else {
                        match entity {
                            "amp" => Some('&'),
                            "lt" => Some('<'),
                            "gt" => Some('>'),
                            "quot" => Some('"'),
                            "apos" => Some('\''),
                            "nbsp" => Some(' '),
                            "ndash" => Some('–'),
                            "mdash" => Some('—'),
                            "lsquo" => Some('‘'),
                            "rsquo" => Some('’'),
                            "ldquo" => Some('“'),
                            "rdquo" => Some('”'),
                            "hellip" => Some('…'),
                            "minus" => Some('−'),
                            "times" => Some('×'),
                            "divide" => Some('÷'),
                            "plusmn" => Some('±'),
                            "deg" => Some('°'),
                            "bull" => Some('•'),
                            "prime" => Some('′'),
                            "Prime" => Some('″'),
                            "frac12" => Some('½'),
                            "frac14" => Some('¼'),
                            "frac34" => Some('¾'),
                            "copy" => Some('©'),
                            "reg" => Some('®'),
                            "trade" => Some('™'),
                            "euro" => Some('€'),
                            "pound" => Some('£'),
                            "yen" => Some('¥'),
                            "cent" => Some('¢'),
                            "sect" => Some('§'),
                            "para" => Some('¶'),
                            "middot" => Some('·'),
                            "larr" => Some('←'),
                            "uarr" => Some('↑'),
                            "rarr" => Some('→'),
                            "darr" => Some('↓'),
                            "harr" => Some('↔'),
                            "crarr" => Some('↵'),
                            _ => None,
                        }
                    };

                    if let Some(ch) = decoded_char {
                        result.push(ch);
                        let end_pos = i + semicolon_idx;
                        while let Some(&(next_i, _)) = chars.peek() {
                            if next_i <= end_pos {
                                chars.next();
                            } else {
                                break;
                            }
                        }
                        continue;
                    }
                }
        }
        result.push(c);
    }

    std::borrow::Cow::Owned(result)
}

pub fn strip_html_tags(input: &str) -> String {
    let mut in_tag = false;
    let mut in_comment = false;
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars();

    while let Some(c) = chars.next() {
        if in_comment {
            if c == '-' && chars.as_str().starts_with("->") {
                chars.next();
                chars.next();
                in_comment = false;
            }
            continue;
        }
        if c == '<' {
            if chars.as_str().starts_with("!--") {
                chars.next();
                chars.next();
                chars.next();
                in_comment = true;
            } else {
                in_tag = true;
            }
            continue;
        }
        if c == '>' && in_tag {
            in_tag = false;
            continue;
        }
        if !in_tag {
            result.push(c);
        }
    }

    let decoded = decode_html_entities(&result);
    decoded.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn to_superscript_char(c: char) -> char {
    match c {
        '0' => '⁰',
        '1' => '¹',
        '2' => '²',
        '3' => '³',
        '4' => '⁴',
        '5' => '⁵',
        '6' => '⁶',
        '7' => '⁷',
        '8' => '⁸',
        '9' => '⁹',
        '+' => '⁺',
        '-' | '−' => '⁻',
        '=' => '⁼',
        '(' | '[' => '⁽',
        ')' | ']' => '⁾',
        'a' => 'ᵃ',
        'b' => 'ᵇ',
        'c' => 'ᶜ',
        'd' => 'ᵈ',
        'e' => 'ᵉ',
        'f' => 'ᶠ',
        'g' => 'ᵍ',
        'h' => 'ʰ',
        'i' => 'ⁱ',
        'j' => 'ʲ',
        'k' => 'ᵏ',
        'l' => 'ˡ',
        'm' => 'ᵐ',
        'n' => 'ⁿ',
        'o' => 'ᵒ',
        'p' => 'ᵖ',
        'r' => 'ʳ',
        's' => 'ˢ',
        't' => 'ᵗ',
        'u' => 'ᵘ',
        'v' => 'ᵛ',
        'w' => 'ʷ',
        'x' => 'ˣ',
        'y' => 'ʸ',
        'z' => 'ᶻ',
        'A' => 'ᴬ',
        'B' => 'ᴮ',
        'D' => 'ᴰ',
        'E' => 'ᴱ',
        'G' => 'ᴳ',
        'H' => 'ᴴ',
        'I' => 'ᴵ',
        'J' => 'ᴶ',
        'K' => 'ᴷ',
        'L' => 'ᴸ',
        'M' => 'ᴹ',
        'N' => 'ᴺ',
        'O' => 'ᴼ',
        'P' => 'ᴾ',
        'R' => 'ᴿ',
        'T' => 'ᵀ',
        'U' => 'ᵁ',
        'W' => 'ᵂ',
        other => other,
    }
}

pub fn to_superscript_str(s: &str) -> String {
    s.chars().map(to_superscript_char).collect()
}

pub fn to_subscript_char(c: char) -> char {
    match c {
        '0' => '₀',
        '1' => '₁',
        '2' => '₂',
        '3' => '₃',
        '4' => '₄',
        '5' => '₅',
        '6' => '₆',
        '7' => '₇',
        '8' => '₈',
        '9' => '₉',
        '+' => '₊',
        '-' | '−' => '₋',
        '=' => '₌',
        '(' | '[' => '₍',
        ')' | ']' => '₎',
        'a' => 'ₐ',
        'e' => 'ₑ',
        'h' => 'ₕ',
        'i' => 'ᵢ',
        'j' => 'ⱼ',
        'k' => 'ₖ',
        'l' => 'ₗ',
        'm' => 'ₘ',
        'n' => 'ₙ',
        'o' => 'ₒ',
        'p' => 'ₚ',
        'r' => 'ᵣ',
        's' => 'ₛ',
        't' => 'ₜ',
        'u' => 'ᵤ',
        'v' => 'ᵥ',
        'x' => 'ₓ',
        other => other,
    }
}

pub fn to_subscript_str(s: &str) -> String {
    s.chars().map(to_subscript_char).collect()
}
