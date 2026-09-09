use crate::api::summary::fetch_random_summary;
use crate::cli::output::{article_url, bold, dim, italic};
use serde::Serialize;

#[derive(Debug, Clone)]
pub struct RandomArgs {
    pub json: bool,
}

#[derive(Serialize)]
struct RandomJsonOutput {
    title: String,
    description: Option<String>,
    extract: Option<String>,
    url: String,
}

pub fn run_random(args: &RandomArgs) -> Result<(), Box<dyn std::error::Error>> {
    let (agent, timeout_secs) = super::client();
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
