complete -c wikid -f

complete -c wikid -n "__fish_use_subcommand" -a search -d "search wikipedia articles"
complete -c wikid -n "__fish_use_subcommand" -a random -d "fetch and print a random article summary"
complete -c wikid -n "__fish_use_subcommand" -a help -d "print help information"
complete -c wikid -n "__fish_use_subcommand" -a version -d "print version"
complete -c wikid -n "__fish_use_subcommand" -a completions -d "generate shell completions"

complete -c wikid -n "__fish_seen_subcommand_from search" -s l -l limit -d "maximum number of results" -r
complete -c wikid -n "__fish_seen_subcommand_from search" -s j -l json -d "output results as json"

complete -c wikid -n "__fish_seen_subcommand_from random" -s j -l json -d "output results as json"

complete -c wikid -n "__fish_seen_subcommand_from completions" -a fish -d "fish shell"
