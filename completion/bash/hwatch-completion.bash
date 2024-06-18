_hwatch() {
    local cur prev opts
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    opts="-b --batch -B --beep --border --with-scrollbar --mouse -c --color -r --reverse -C --compress -t --no-title -N --line-number --no-help-banner -x --exec -O --diff-output-only -A --aftercommand -l --logfile -s --shell -n --interval -L --limit --tab-size -d --differences -o --output -K --keymap -h --help -V --version"

    if [[ ${cur} == -* ]]; then
        COMPREPLY=( $(compgen -W "${opts}" -- ${cur}) )
        return 0
    fi
}

complete -F _hwatch hwatch
