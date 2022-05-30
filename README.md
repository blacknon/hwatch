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

    hwatch 0.3.5
    blacknon <blacknon@orebibou.com>
    A modern alternative to the watch command, records the differences in execution results and can check this differences
    at after.

    USAGE:
        hwatch [FLAGS] [OPTIONS] <command>...

    FLAGS:
        -c, --color          interpret ANSI color and style sequences
        -d, --differences    highlight changes between updates
        -N, --line-number    show line number
        -x, --exec           Run the command directly, not through the shell. Much like the `-x` option of the watch
                             command.
        -h, --help           Prints help information
        -V, --version        Prints version information

    OPTIONS:
        -l, --logfile <logfile>        logging file
        -s, --shell <shell_command>    shell to use at runtime. can  also insert the command to the location specified by
                                       {COMMAND}. [default: sh -c]
        -n, --interval <interval>      seconds to wait between updates [default: 2]

    ARGS:
        <command>...

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

```bash
hwatch -n 10 command...
```


### logging output

```bash
hwatch -n 10 -l hwatch_log.json command...
```

### Use shell function

```bash

```

### history filtering


### diff view
