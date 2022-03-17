complete -c hwatch -s h -l help -d 'show help'
complete -c hwatch -s V -l version -d 'show version'
complete -c hwatch -s c -l color -d 'interpret ANSI color and style sequences'
complete -c hwatch -s d -l differences -d 'highlight changes between updates'
complete -c hwatch -s l -l logfile -d 'logging file'
complete -c hwatch -s n -l interval -x -d 'seconds to wait between updates'
complete -c hwatch -xa '(__fish_complete_subcommand -- -n --interval)' -d Command
