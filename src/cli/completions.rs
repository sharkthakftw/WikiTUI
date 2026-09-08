pub const FISH_COMPLETIONS: &str = include_str!("../../completions/wikid.fish");

pub fn run_completions(shell: &str) -> Result<(), Box<dyn std::error::Error>> {
    match shell {
        "fish" => {
            print!("{}", FISH_COMPLETIONS);
            Ok(())
        }
        other => {
            eprintln!("Unsupported shell: {}", other);
            eprintln!("Supported shells: fish");
            std::process::exit(1);
        }
    }
}
