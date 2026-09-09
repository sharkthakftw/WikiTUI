use crate::api::random::fetch_random_article;
use crate::api::summary::fetch_random_summary;
use crate::cli::output::{article_url, bold, dim, is_stdout_terminal, italic, render_line};
use serde::Serialize;

#[derive(Debug, Clone)]
pub struct RandomArgs {
    pub json: bool,
    pub full: bool,
}

#[derive(Serialize)]
struct RandomJsonOutput {
    title: String,
    description: Option<String>,
    extract: Option<String>,
    url: String,
}

pub fn run_random(args: &RandomArgs) -> Result<(), Box<dyn std::error::Error>> {
    let (agent, config) = super::client();
    let timeout_secs = config.network.timeout;

    if args.full {
        let (title, content) = fetch_random_article(
            &agent,
            timeout_secs,
            config.network.offline_cache,
            config.network.cache_lifetime,
        )?;
        let width = if is_stdout_terminal() {
            crossterm::terminal::size()
                .map(|(w, _)| w as usize)
                .unwrap_or(80)
                .max(20)
        } else {
            80
        };
        let doc = crate::parser::parse_wikipedia_html(
            &content,
            width,
            config.reader.show_footnotes,
            config.reader.show_external_links,
            config.reader.heading_marker,
            config.reader.code_line_numbers,
            config.ui.icons,
            false,
            0,
        );

        if args.json {
            let full_text = doc
                .lines
                .iter()
                .map(|line| {
                    line.spans
                        .iter()
                        .map(|s| s.content.as_ref())
                        .collect::<String>()
                })
                .collect::<Vec<_>>()
                .join("\n");
            let url = article_url(&title);
            let out = RandomJsonOutput {
                title,
                description: None,
                extract: Some(full_text),
                url,
            };
            println!("{}", serde_json::to_string_pretty(&out)?);
            return Ok(());
        }

        println!("{}", bold(&title));
        println!();
        for line in &doc.lines {
            println!("{}", render_line(line));
        }
        return Ok(());
    }

    let (title, description, extract) = fetch_random_summary(&agent, timeout_secs)?;

    if args.json {
        let url = article_url(&title);
        let out = RandomJsonOutput {
            title,
            description,
            extract,
            url,
        };
        println!("{}", serde_json::to_string_pretty(&out)?);
        return Ok(());
    }

    println!("{}", bold(&title));
    if let Some(desc) = &description {
        if !desc.trim().is_empty() {
            println!("{}", italic(&dim(desc.trim())));
        }
    }
    if let Some(ext) = &extract {
        if !ext.trim().is_empty() {
            println!();
            println!("{}", ext.trim());
        }
    }

    Ok(())
}
