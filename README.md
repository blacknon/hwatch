hwatch
======

hwatch - alternative watch command.

<p align="center">
<img src="./img/hwatch.gif" />
</p>

## Description

`hwatch` is a alternative **watch** command.
That records the result of command execution and can display it history and diffs.

### Features

- Can keep the history when the difference, occurs and check it later.
- Can check the difference in the history. The display method can be changed in real time.
- Can output the execution result as log (json format).
- Custom keymaps are available.
- Support ANSI color code.
- Execution result can be scroll.
- Not only as a TUI application, but also to have the differences output as standard output.
- If a difference occurs, you can have the specified command additionally executed.

## Install

### macOS (brew)

    brew install hwatch

### macOS (MacPorts)

    sudo port install hwatch

### Arch Linux (AUR)

    paru -S hwatch

### Cargo Install

    cargo install hwatch

## Usage

### Command

        $ hwatch --help
        A modern alternative to the watch command, records the differences in execution results and can check this differences at after.

        Usage: hwatch [OPTIONS] [command]...

        Arguments:
          [command]...

        Options:
          -b, --batch                         output exection results to stdout
          -B, --beep                          beep if command has a change result
              --border                        Surround each pane with a border frame
              --with-scrollbar                When the border option is enabled, display scrollbar on the right side of watch pane.
              --mouse                         enable mouse wheel support. With this option, copying text with your terminal may be harder. Try holding the Shift key.
          -c, --color                         interpret ANSI color and style sequences
          -r, --reverse                       display text upside down.
          -C, --compress                      Compress data in memory.
          -t, --no-title                      hide the UI on start. Use `t` to toggle it.
          -N, --line-number                   show line number
              --no-help-banner                hide the "Display help with h key" message
          -x, --exec                          Run the command directly, not through the shell. Much like the `-x` option of the watch command.
          -O, --diff-output-only              Display only the lines with differences during `line` diff and `word` diff.
          -A, --aftercommand <after_command>  Executes the specified command if the output changes. Information about changes is stored in json format in environment variable ${HWATCH_DATA}.
          -l, --logfile [<logfile>]           logging file
          -s, --shell <shell_command>         shell to use at runtime. can  also insert the command to the location specified by {COMMAND}. [default: "sh -c"]
          -n, --interval <interval>           seconds to wait between updates [default: 2]
          -L, --limit <limit>                 Set the number of history records to keep. only work in watch mode. Set `0` for unlimited recording. (default: 5000) [default: 5000]
              --tab-size <tab_size>           Specifying tab display size [default: 4]
          -d, --differences [<differences>]   highlight changes between updates [possible values: none, watch, line, word]
          -o, --output [<output>]             Select command output. [default: output] [possible values: output, stdout, stderr]
          -K, --keymap <keymap>               Add keymap
          -h, --help                          Print help
          -V, --version                       Print version

### Keybind

Watch mode keybind(Default).

| Key                                  | Action                                                      |
|--------------------------------------|-------------------------------------------------------------|
| <kbd>↑</kbd>, <kbd>↓</kbd>           | move selected screen(history/watch).                        |
| <kbd>pageup</kbd>, <kbd>pagedn</kbd> | move selected screen(history/watch).                        |
| <kbd>home</kbd>, <kbd>end</kbd>      | move selected screen(history/watch).                        |
| <kbd>Tab</kbd>                       | toggle select screen(history/watch).                        |
| <kbd>←</kbd>                         | select watch screen.                                        |
| <kbd>→</kbd>                         | select history screen.                                      |
| <kbd>H</kbd>                         | show help window.                                           |
| <kbd>B</kbd>                         | toggle enable/disable border.                               |
| <kbd>S</kbd>                         | toggle enable/disable border scrollbar.                     |
| <kbd>C</kbd>                         | toggle color.                                               |
| <kbd>N</kbd>                         | switch line number display.                                 |
| <kbd>R</kbd>                         | toggle reverse mode.                                        |
| <kbd>M</kbd>                         | toggle mouse support.                                       |
| <kbd>D</kbd>                         | switch diff mode.                                           |
| <kbd>T</kbd>                         | toggle the UI (history pane and header).                    |
| <kbd>Backspace</kbd>                 | toggle the history pane.                                    |
| <kbd>Q</kbd>                         | exit hwatch.                                                |
| <kbd>0</kbd>                         | disable diff.                                               |
| <kbd>1</kbd>                         | switch watch type diff.                                     |
| <kbd>2</kbd>                         | switch line type diff.                                      |
| <kbd>3</kbd>                         | switch word type diff.                                      |
| <kbd>O</kbd>                         | switch output mode(output->stdout->stderr).                 |
| <kbd>Shift</kbd>+<kbd>O</kbd>        | show only lines with differences(line/word diff mode only). |
| <kbd>Shift</kbd>+<kbd>S</kbd>        | show summary infomation in history.                         |
| <kbd>F1</kbd>                        | only stdout print.                                          |
| <kbd>F2</kbd>                        | only stderr print.                                          |
| <kbd>F3</kbd>                        | print output.                                               |
| <kbd>+</kbd>                         | increase interval.                                          |
| <kbd>-</kbd>                         | decrease interval.                                          |
| <kbd>/</kbd>                         | filter history by string.                                   |
| <kbd>*</kbd>                         | filter history by regex.                                    |
| <kbd>Esc</kbd>                       | unfiltering.                                                |
| <kbd>Ctrl</kbd>+<kbd>c</kbd>         | cancel.                                                     |

#### Custom keybind

Can customize key bindings by using the `-K` Option.
Write it in the format `keybind=funciton`.

```bash
hwatch -K ctrl-p=history_pane_up -K ctrl-n=history_pane_down command...
```

Keybind functions that can be specified are as follows.

| function                 | description                              |
|--------------------------|------------------------------------------|
| up                       | Move up                                  |
| watch_pane_up            | Move up in watch pane                    |
| history_pane_up          | Move up in history pane                  |
| down                     | Move down                                |
| watch_pane_down          | Move down in watch pane                  |
| history_pane_down        | Move down in history pane                |
| page_up                  | Move page up                             |
| watch_pane_page_up       | Move page up in watch pane               |
| history_pane_page_up     | Move page up in history pane             |
| page_down                | Move page down                           |
| watch_pane_page_down     | Move page down in watch pane             |
| history_pane_page_down   | Move page down in history pane           |
| move_top                 | Move top                                 |
| watch_pane_move_top      | Move top in watch pane                   |
| history_pane_move_top    | Move top in history pane                 |
| move_end                 | Move end                                 |
| watch_pane_move_end      | Move end in watch pane                   |
| history_pane_move_end    | Move end in history pane                 |
| toggle_forcus            | Toggle forcus window                     |
| forcus_watch_pane        | Forcus watch pane                        |
| forcus_history_pane      | Forcus history pane                      |
| quit                     | Quit hwatch                              |
| reset                    | filter reset                             |
| cancel                   | Cancel                                   |
| help                     | Show and hide help window                |
| toggle_color             | Toggle enable/disable ANSI Color         |
| toggle_line_number       | Toggle enable/disable Line Number        |
| toggle_reverse           | Toggle enable/disable text reverse       |
| toggle_mouse_support     | Toggle enable/disable mouse support      |
| toggle_view_pane_ui      | Toggle view header/history pane          |
| toggle_view_header_pane  | Toggle view header pane                  |
| toggle_view_history_pane | Toggle view history pane                 |
| toggle_border            | Toggle enable/disable border             |
| toggle_scroll_bar        | Toggle enable/disable scroll bar         |
| toggle_diff_mode         | Toggle diff mode                         |
| set_diff_mode_plane      | Set diff mode plane                      |
| set_diff_mode_watch      | Set diff mode watch                      |
| set_diff_mode_line       | Set diff mode line                       |
| set_diff_mode_word       | Set diff mode word                       |
| set_diff_only            | Set diff line only (line/word diff only) |
| toggle_output_mode       | Toggle output mode                       |
| set_output_mode_output   | Set output mode output                   |
| set_output_mode_stdout   | Set output mode stdout                   |
| set_output_mode_stderr   | Set output mode stderr                   |
| togge_history_summary    | Toggle history summary                   |
| interval_plus            | Interval +0.5sec                         |
| interval_minus           | Interval -0.5sec                         |
| change_filter_mode       | Change filter mode                       |
| change_regex_filter_mode | Change regex filter mode                 |


## Configuration

If you always want to use some command-line options, you can set them in the
`HWATCH` environment variable. For example, if you use `bash`, you can add
the following to your `.bashrc`:

```bash
export HWATCH="--no-title --color --no-help-banner --border --with-scrollbar"
```

## Example

### interval 10 second

Use the -n option to specify the command execution interval.

```bash
hwatch -n 3 command...
```

<p align="center">
<img src="./img/interval.gif" />
</p>

### logging output

The command execution result can be output as a log in json format.

```bash
hwatch -n 3 -l hwatch_log.json command...
```

When you check the json log, you can easily check it by using [this script](https://gist.github.com/blacknon/551e52dce1651d2510162def5a0da1f0).

### Use shell function

If you want the shell function to be executed periodically, you can specify the shell command to be executed with -s as follows.

```bash
# bash
hwatch -n 3 -s 'bash -c "source ~/.bashrc"; {COMMAND}' command...

# zsh
hwatch -n 3 -s 'zsh -c "source ~/.zshrc"; {COMMAND}' command...
```

### ANSI Color code

If you want to see output colored with ANSI color code, enable color mode.

To enable color mode, run hwatch with the `-c` option.
Alternatively, you can enable / disable the color mode with the <kbd>C</kbd> key during execution.

```bash
hwatch -n 3 -c command...
```

### diff view

To enable color mode, run hwatch with the `-d` option.

There are several "diff modes" available.
Switching can be done with the <kbd>D</kbd> key.

```bash
hwatch -n 3 -d command...
```

#### watch diff

<p align="center">
<img src="./img/watch_diff.png" />
</p>

#### line diff

<p align="center">
<img src="./img/line_diff.png" />
</p>

#### word diff

<p align="center">
<img src="./img/word_diff.png" />
</p>

### history filtering

You can filter history as a string with <kbd>/</kbd> key and as a regular expression with <kbd>*</kbd> key.

### run batch mode

You can have command diffs output directly to stdout instead with `-b` option of getting them as a TUI app.

```bash
hwatch -b command...
```

## Alternatives
- The original [`watch`](https://man7.org/linux/man-pages/man1/watch.1.html);
  the newest version seems to be distributed as a part of
  [`procps`](https://gitlab.com/procps-ng/procps).
- [Viddy](https://github.com/sachaos/viddy).
- [sasqwatch](https://github.com/fabio42/sasqwatch)
