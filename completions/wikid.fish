complete -c wikid -f

complete -c wikid -s r -l random -d "open the tui with a random article"
complete -c wikid -s h -l help -d "print help information"
complete -c wikid -s v -l version -d "print version"

complete -c wikid -n "__fish_use_subcommand" -a search -d "search wikipedia articles"
complete -c wikid -n "__fish_use_subcommand" -a summary -d "fetch and print an article summary"
complete -c wikid -n "__fish_use_subcommand" -a category -d "fetch article categories"
complete -c wikid -n "__fish_use_subcommand" -a random -d "fetch and print a random article summary"
complete -c wikid -n "__fish_use_subcommand" -a help -d "print help information"
complete -c wikid -n "__fish_use_subcommand" -a version -d "print version"
complete -c wikid -n "__fish_use_subcommand" -a completions -d "generate shell completions"

complete -c wikid -n "__fish_seen_subcommand_from search" -s l -l limit -d "maximum number of results" -r
complete -c wikid -n "__fish_seen_subcommand_from search" -s j -l json -d "output results as json"

complete -c wikid -n "__fish_seen_subcommand_from summary" -s j -l json -d "output results as json"

complete -c wikid -n "__fish_seen_subcommand_from category" -s j -l json -d "output results as json"

complete -c wikid -n "__fish_seen_subcommand_from random" -s f -l full -d "fetch the whole article"
complete -c wikid -n "__fish_seen_subcommand_from random" -s j -l json -d "output results as json"

complete -c wikid -n "__fish_seen_subcommand_from completions" -a fish -d "fish shell"
