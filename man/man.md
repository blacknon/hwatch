% hwatch(1) Version 0.1.4 | alternative watch command.

NAME
====

**hwatch** - alternative watch command.

SYNOPSIS
========

| **hwatch** \[*options*] *command*

DESCRIPTION
===========

**hwatch** is like *watch* command, repeatedly executes a *command* and displays its output.
However, the output results can be scrolled and displayed.
In addition, the difference of the execution result is recorded with the time stamp, and it can be checked later.
When checking, it is also possible to display the diff with the previous difference together.

If you specify it with options, the execution result can be recorded as a log with json.


Flags
-----

-h, --help

:   Prints help information


-V, --version

:   Prints version information


-c, --color

:   Interpret ANSI colors and style sequences and display in color. It can be changed later by key binding.


-d, --differences

:   highlight changes between updates. The diff specified by this flag is similar to the *watch* command. It can be changed later by key binding.



Options
-------

-n, --interval *seconds*

:   Specify update interval. The command will not allow quicker than **0.001** second interval, in which the smaller values are converted. Both '.' and ',' work for any locales.


-l, --logfile *logfile*

:   Output the command execution result and its time as a log in json. The execution results that are recorded are only those that differ from the previous execution results.


KEYBINDS
========


ENVIRONMENT
===========

**DEFAULT_HELLO_DEDICATION**

:   The default dedication if none is given. Has the highest precedence
    if a dedication is not supplied on the command line.

NOTES
=====


EXAMPLES
========


BUGS
====

See GitHub Issues: <https://github.com/blacknon/hwatch/issues>

AUTHOR
======

Blacknon <blacknon@orebibou.com>
