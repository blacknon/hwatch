hwatch
====

hwatch - alternative watch command.

<p align="center">
<img src="./tty.gif" />
</p>

## Description

Record the execution result of the command, and after can view this.

## Install

### MacOS(brew)

    brew tap blacknon/hwatch
    brew install hwatch

### Cargo Install

    git clone github.com/blacknon/hwatch
    cd hwatch
    cargo install

## Usage

    hwatch 0.1.1
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
        -n, --interval <interval>    seconds to wait between updates [default: 2]
    
    ARGS:
        <command>...

watch window keybind

    Arrow up/down  ... move selected screen(history/watch).
    Tab            ... toggle select screen(history/watch).
    c              ... toggle color.
    d              ... switch diff mode.
    0              ... disable diff.
    1              ... switch watch type diff.
    2              ... switch line type diff.
    F1             ... only stdout print.
    F2             ... only stderr print.
    F3             ... print output.
