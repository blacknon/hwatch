hwatch
======

hwatch - alternative watch command.

<p align="center">
<img src="./img/tty.gif" />
</p>

## Description

`hwatch` is a alternative **watch** command.
That records the result of command execution and can display it history and diffs.

### Features

- Can keep the history when the difference, occurs and check it later.
- Can check the difference in the history. The display method can be changed in real time.
- Can output the execution result as log (json format).
- Support ANSI color code.
- Execution result can be scroll.

## Install

### macOS (brew)

    brew tap blacknon/hwatch
    brew install hwatch

### macOS (MacPorts)

    sudo port install hwatch

### Cargo Install

    cargo install hwatch

## Usage

    hwatch 0.3.6
    blacknon <blacknon@orebibou.com>
    A modern alternative to the watch command, records the differences in execution results and can
    check this differences at after.

    USAGE:
        hwatch [OPTIONS] <command>...

    ARGS:
        <command>...

    OPTIONS:
        -c, --color                    interpret ANSI color and style sequences
        -d, --differences              highlight changes between updates
        -N, --line-number              show line number
        -x, --exec                     Run the command directly, not through the shell. Much like the
                                       `-x` option of the watch command.
        -l, --logfile <logfile>        logging file
        -s, --shell <shell_command>    shell to use at runtime. can  also insert the command to the
                                       location specified by {COMMAND}. [default: "sh -c"]
        -n, --interval <interval>      seconds to wait between updates [default: 2]
        -h, --help                     Print help information
        -V, --version                  Print version information


watch window keybind

- <kbd>↑</kbd>, <kbd>↓</kbd>  ... move selected screen(history/watch).
- <kbd>H</kbd>   ... show help window.
- <kbd>C</kbd>   ... toggle color.
- <kbd>D</kbd>   ... switch diff mode.
- <kbd>N</kbd>   ... switch line number display.
- <kbd>Q</kbd>   ... exit hwatch.
- <kbd>0</kbd>   ... disable diff.
- <kbd>1</kbd>   ... switch watch type diff.
- <kbd>2</kbd>   ... switch line type diff.
- <kbd>3</kbd>   ... switch word type diff.
- <kbd>F1</kbd>  ... only stdout print.
- <kbd>F2</kbd>  ... only stderr print.
- <kbd>F3</kbd>  ... print output.
- <kbd>Tab</kbd> ... toggle select screen(history/watch).
- <kbd>/</kbd>   ... filter history by string.
- <kbd>*</kbd>   ... filter history by regex.
- <kbd>Esc</kbd> ... unfiltering.


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

<p align="center">
<img src="./img/shell_function.gif" />
</p>


### ANSI Color code

If you want to see output colored with ANSI color code, enable color mode.

To enable color mode, run hwatch with the `-c` option.
Alternatively, you can enable / disable the color mode with the <kbd>C</kbd> key during execution.

```bash
hwatch -n 3 -c command...
```

<p align="center">
<img src="./img/ansi_color.gif" />
</p>


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
