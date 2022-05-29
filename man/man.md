% hwatch(1) Version 0.3.5 | A modern alternative to the watch command, records the differences in execution results and can check this differences at after.

NAME
====

**hwatch** - A modern alternative to the watch command, records the differences in execution results and can check this differences at after.

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

:   Highlight changes between updates. The diff specified by this flag is similar to the *watch* command. It can be changed later by key binding.


-N, --line-number

:   Show line number.



Options
-------

-n, --interval *seconds*

:   Specify update interval. The command will not allow quicker than **0.001** second interval, in which the smaller values are converted. Both '.' and ',' work for any locales.


-l, --logfile *logfile*

:   Output the command execution result and its time as a log in json. The execution results that are recorded are only those that differ from the previous execution results.


KEYBINDS
========

**hwatch** uses *Keybind* for operations on the command execution screen.

h

:   Show help message. Press the *h* key again to return to the previous screen.


q

:   Exit hwatch.


c

:   Interprets ANSI colors and style sequences and displays them in color. This is the same as the *-c(--color)* option. Press the *c* key again to return to the original.


d

:   Highlight changes between updates. The diff specified by this flag is similar to the *watch* command. This is the same as the *-d(--differences)* option. You can switch the diff mode by pressing the *d* key. The *d* key toggles these in order. Use the *0*, *1*, and *2* keys to switch directly to each mode.

      *plane* ... Do not show diff (default).

      *watch* ... Diff like watch command. Specifying the *-d* option applies this mode.

      *line*  ... Can be done diff in line units.

      *word*  ... Can be done diff in line word units.


0

:   Switch diff mode to *plane*.


1

:   Switch diff mode to *watch*.


2

:   Switch diff mode to *line*.


3

:   Switch diff mode to *word*.


F1

:   Display only *Stdout*.


F2

:   Display only *Stderr*.


F3

:   Display *Stdout* and *Stderr*.


Tab

:   Switch the target(*history* or *watch* pad). The target is operated with the *up* and *down* keys.


/

:   Filter diffs by keyword.


\*

:   Filter diffs by regex.



BUGS
====

See GitHub Issues: <https://github.com/blacknon/hwatch/issues>

AUTHOR
======

Blacknon <blacknon@orebibou.com>
