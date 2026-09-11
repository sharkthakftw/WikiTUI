use crate::api::random::fetch_random_article;
use crate::api::summary::fetch_random_summary;
use crate::cli::output::{bold, is_stdout_terminal, print_summary, render_line};

#[derive(Debug, Clone)]
pub struct RandomArgs {
    pub json: bool,
    pub full: bool,
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
            return print_summary(&title, None, Some(&full_text), true);
        }

        println!("{}", bold(&title));
        println!();
        for line in &doc.lines {
            println!("{}", render_line(line));
        }
        return Ok(());
    }

    let (title, description, extract) = fetch_random_summary(&agent, timeout_secs)?;
    print_summary(&title, description.as_deref(), extract.as_deref(), args.json)
}
