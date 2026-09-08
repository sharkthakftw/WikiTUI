pub mod output;
pub mod search;

use search::SearchArgs;

pub enum CliCommand {
    Search(SearchArgs),
    Help,
    Version,
}

pub fn parse_args(args: &[String]) -> Option<CliCommand> {
    if args.is_empty() {
        return None;
    }

    let first = args[0].as_str();

    match first {
        "help" => Some(CliCommand::Help),
        "version" => Some(CliCommand::Version),
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
    println!("    search <query>  search wikipedia articles");
    println!("    help            print help information");
    println!("    version         print version");
    println!();
    println!("SEARCH OPTIONS:");
    println!("    -l, --limit <n> maximum number of results (default: 10)");
    println!("    -j, --json      output results as json");
}
