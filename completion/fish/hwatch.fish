complete -c hwatch -s A -l aftercommand -d 'Executes the specified command if the output changes. Information about changes is stored in json format in environment variable ${HWATCH_DATA}.' -r -f -a "(__fish_complete_command)"
complete -c hwatch -s l -l logfile -d 'logging file' -r -F
complete -c hwatch -s s -l shell -d 'shell to use at runtime. can  also insert the command to the location specified by {COMMAND}.' -r -f -a "(__fish_complete_command)"
complete -c hwatch -s n -l interval -d 'seconds to wait between updates' -r
complete -c hwatch -s L -l limit -d 'Set the number of history records to keep. only work in watch mode. Set `0` for unlimited recording. (default: 5000)' -r
complete -c hwatch -l tab-size -d 'Specifying tab display size' -r
complete -c hwatch -s d -l differences -d 'highlight changes between updates' -r -f -a "{none\t'',watch\t'',line\t'',word\t''}"
complete -c hwatch -s o -l output -d 'Select command output.' -r -f -a "{output\t'',stdout\t'',stderr\t''}"
complete -c hwatch -s K -l keymap -d 'Add keymap' -r
complete -c hwatch -s b -l batch -d 'output exection results to stdout'
complete -c hwatch -s B -l beep -d 'beep if command has a change result'
complete -c hwatch -l border -d 'Surround each pane with a border frame'
complete -c hwatch -l with-scrollbar -d 'When the border option is enabled, display scrollbar on the right side of watch pane.'
complete -c hwatch -l mouse -d 'enable mouse wheel support. With this option, copying text with your terminal may be harder. Try holding the Shift key.'
complete -c hwatch -s c -l color -d 'interpret ANSI color and style sequences'
complete -c hwatch -s r -l reverse -d 'display text upside down.'
complete -c hwatch -s C -l compress -d 'Compress data in memory. Note: If the output of the command is small, you may not get the desired effect.'
complete -c hwatch -s t -l no-title -d 'hide the UI on start. Use `t` to toggle it.'
complete -c hwatch -s N -l line-number -d 'show line number'
complete -c hwatch -l no-help-banner -d 'hide the "Display help with h key" message'
complete -c hwatch -s x -l exec -d 'Run the command directly, not through the shell. Much like the `-x` option of the watch command.'
complete -c hwatch -s O -l diff-output-only -d 'Display only the lines with differences during `line` diff and `word` diff.'
complete -c hwatch -s h -l help -d 'Print help'
complete -c hwatch -s V -l version -d 'Print version'
