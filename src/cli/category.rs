use crate::api::category::fetch_article_categories;
use serde::Serialize;

#[derive(Debug, Clone)]
pub struct CategoryArgs {
    pub title: String,
    pub json: bool,
}

#[derive(Serialize)]
struct CategoryJsonOutput {
    title: String,
    categories: Vec<String>,
}

pub fn run_category(args: &CategoryArgs) -> Result<(), Box<dyn std::error::Error>> {
    let (agent, config) = super::client();
    let (title, categories) =
        fetch_article_categories(&agent, &args.title, config.network.timeout)?;

    if args.json {
        let out = CategoryJsonOutput {
            title,
            categories,
        };
        println!("{}", serde_json::to_string_pretty(&out)?);
        return Ok(());
    }

    for cat in &categories {
        println!("{}", cat);
    }

    Ok(())
}
