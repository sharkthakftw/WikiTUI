use crate::api::summary::fetch_summary;
use crate::cli::output::print_summary;

#[derive(Debug, Clone)]
pub struct SummaryArgs {
    pub title: String,
    pub json: bool,
}

pub fn run_summary(args: &SummaryArgs) -> Result<(), Box<dyn std::error::Error>> {
    let (agent, config) = super::client();
    let (title, description, extract) =
        fetch_summary(&agent, &args.title, config.network.timeout)?;

    print_summary(&title, description.as_deref(), extract.as_deref(), args.json)
}
