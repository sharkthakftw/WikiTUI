use crate::api::search::search_wikipedia;
use crate::cli::output::{article_url, bold, cyan, dim, yellow};
use serde::Serialize;

#[derive(Debug, Clone)]
pub struct SearchArgs {
    pub query: String,
    pub limit: usize,
    pub json: bool,
}

#[derive(Serialize)]
struct SearchJsonItem {
    title: String,
    description: String,
    has_audio: bool,
    url: String,
}

pub fn run_search(args: &SearchArgs) -> Result<(), Box<dyn std::error::Error>> {
    let (agent, timeout_secs) = super::client();
    let results = search_wikipedia(&agent, &args.query, args.limit, timeout_secs)?;

    if args.json {
        let json_items: Vec<SearchJsonItem> = results
            .into_iter()
            .map(|item| {
                let url = article_url(&item.title);
                SearchJsonItem {
                    title: item.title,
                    description: item.snippet,
                    has_audio: item.has_audio,
                    url,
                }
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&json_items)?);
        return Ok(());
    }

    if results.is_empty() {
        eprintln!("No results found for {:?}", args.query);
        return Ok(());
    }

    for (idx, item) in results.into_iter().enumerate() {
        let num = format!("{:2}.", idx + 1);
        let audio_badge = if item.has_audio {
            format!(" {}", yellow("[audio]"))
        } else {
            String::new()
        };
        println!(" {} {}{}", cyan(&num), bold(&item.title), audio_badge);
        if !item.snippet.is_empty() {
            println!("     {}", dim(&item.snippet));
        }
    }

    Ok(())
}
