pub mod category;
pub mod completions;
pub mod output;
pub mod random;
pub mod search;
pub mod summary;

use category::CategoryArgs;
use random::RandomArgs;
use search::SearchArgs;
use summary::SummaryArgs;

pub enum CliCommand {
    Search(SearchArgs),
    Random(RandomArgs),
    Summary(SummaryArgs),
    Category(CategoryArgs),
    Completions(String),
    Help,
    Version,
}

pub fn client() -> (ureq::Agent, crate::config::Config) {
    let config = crate::config::Config::load();
    (crate::api::default_agent(), config)
}

fn has_flag(args: &[String], long: &str, short: &str) -> bool {
    args.iter().any(|a| a == long || a == short)
}

pub fn parse_args(args: &[String]) -> Option<CliCommand> {
    if args.is_empty() {
        return None;
    }

    let first = args[0].as_str();

    match first {
        "help" => Some(CliCommand::Help),
        "version" => Some(CliCommand::Version),
        "completions" => {
            let shell = args.get(1).map(|s| s.as_str()).unwrap_or("fish");
            Some(CliCommand::Completions(shell.to_string()))
        }
        "random" => {
            let json = has_flag(&args[1..], "--json", "-j");
            let full = has_flag(&args[1..], "--full", "-f");
            Some(CliCommand::Random(RandomArgs { json, full }))
        }
        "summary" => {
            let mut title_words = Vec::new();
            let mut json = false;
            let mut i = 1;
            while i < args.len() {
                match args[i].as_str() {
                    "--json" | "-j" => {
                        json = true;
                    }
                    other => {
                        title_words.push(other.to_string());
                    }
                }
                i += 1;
            }
            if title_words.is_empty() {
                eprintln!("Error: 'summary' requires an article title");
                eprintln!("Usage: wikid summary [OPTIONS] <title>");
                std::process::exit(1);
            }
            Some(CliCommand::Summary(SummaryArgs {
                title: title_words.join(" "),
                json,
            }))
        }
        "category" => {
            let mut title_words = Vec::new();
            let mut json = false;
            let mut i = 1;
            while i < args.len() {
                match args[i].as_str() {
                    "--json" | "-j" => {
                        json = true;
                    }
                    other => {
                        title_words.push(other.to_string());
                    }
                }
                i += 1;
            }
            if title_words.is_empty() {
                eprintln!("Error: 'category' requires an article title");
                eprintln!("Usage: wikid category [OPTIONS] <title>");
                std::process::exit(1);
            }
            Some(CliCommand::Category(CategoryArgs {
                title: title_words.join(" "),
                json,
            }))
        }
        "search" => {
            let mut query_words = Vec::new();
            let mut limit = 10usize;
            let mut json = false;
            let mut i = 1;
            while i < args.len() {
                match args[i].as_str() {
                    "--json" | "-j" => {
                        json = true;
                    }
                    "--limit" | "-l" => {
                        if i + 1 < args.len() {
                            if let Ok(l) = args[i + 1].parse::<usize>() {
                                limit = l;
                                i += 1;
                            }
                        }
                    }
                    other => {
                        query_words.push(other.to_string());
                    }
                }
                i += 1;
            }
            if query_words.is_empty() {
                eprintln!("Error: 'search' requires a search query");
                eprintln!("Usage: wikid search [OPTIONS] <query>");
                std::process::exit(1);
            }
            Some(CliCommand::Search(SearchArgs {
                query: query_words.join(" "),
                limit,
                json,
            }))
        }
        _ => None,
    }
}

pub fn run(cmd: CliCommand) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        CliCommand::Search(args) => search::run_search(&args),
        CliCommand::Random(args) => random::run_random(&args),
        CliCommand::Summary(args) => summary::run_summary(&args),
        CliCommand::Category(args) => category::run_category(&args),
        CliCommand::Completions(shell) => completions::run_completions(&shell),
        CliCommand::Help => {
            print_help();
            Ok(())
        }
        CliCommand::Version => {
            println!("wikid {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
    }
}

fn print_help() {
    println!("wikid - a feature-rich terminal wikipedia client");
    println!();
    println!("USAGE:");
    println!("    wikid [COMMAND] [OPTIONS] [ARTICLE]");
    println!();
    println!("COMMANDS:");
    println!("    search <query>      search wikipedia articles");
    println!("    summary <title>     fetch and print an article summary");
    println!("    category <title>    fetch article categories");
    println!("    random              fetch and print a random article summary");
    println!("    completions <shell> generate shell completions (fish)");
    println!("    help                print help information");
    println!("    version             print version");
    println!();
    println!("SEARCH OPTIONS:");
    println!("    -l, --limit <n>     maximum number of results (default: 10)");
    println!("    -j, --json          output results as json");
    println!();
    println!("SUMMARY OPTIONS:");
    println!("    -j, --json          output results as json");
    println!();
    println!("CATEGORY OPTIONS:");
    println!("    -j, --json          output results as json");
    println!();
    println!("RANDOM OPTIONS:");
    println!("    -f, --full          fetch and print the full article");
    println!("    -j, --json          output results as json");
}
