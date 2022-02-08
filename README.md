hwatch
======

hwatch - alternative watch command.

<p align="center">
<img src="./img/tty.gif" />
</p>

## Description

`hwatch` is a alternative watch command. That records the result of command execution and can display it later.

## Install

### MacOS(brew)

    brew tap blacknon/hwatch
    brew install hwatch

### Cargo Install

    cargo install hwatch

## Usage

    hwatch 0.3.0
    blacknon <blacknon@orebibou.com>
    alternative watch command.

    USAGE:
        hwatch [FLAGS] [OPTIONS] <command>...

    FLAGS:
        -c, --color          interpret ANSI color and style sequences
        -d, --differences    highlight changes between updates
        -h, --help           Prints help information
        -V, --version        Prints version information

    OPTIONS:
        -l, --logfile <logfile>      logging file
        -n, --interval <interval>    seconds to wait between updates [default: 2]

    ARGS:
        <command>...



watch window keybind

- <kbd>↑</kbd>, <kbd>↓</kbd>  ... move selected screen(history/watch).
- <kbd>H</kbd>   ... show help window.
- <kbd>C</kbd>   ... toggle color.
- <kbd>D</kbd>   ... switch diff mode.
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
- <kbd>Tab</kbd> ... unfiltering.
