% hwatch(1) Version 0.3.13 | A modern alternative to the watch command, records the differences in execution results and can check this differences at after.

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

-h, \--help

:   Prints help information


-V, \--version

:   Prints version information


-b, \--batch

:   Output exection results to stdout. NOTE: Operations with TUI are not possible in batch mode.


-B, \--beep

:   beep if command has a change result.


\--mouse

:   enable mouse wheel support. With this option, copying text with your terminal may be harder. Try holding the Shift key.


-c, \--color

:   Interpret ANSI colors and style sequences and display in color. It can be changed later by key binding.


-r, \--reverse

:   Display text upside down.


-t, \--no-title

:   Hide the UI on start. Use `t` to toggle it.


-N, \--line-number

:   Show line number.


\--no-help-banner

:   Hide the "Display help with h key" message


-e, \--exec

:   Run the command directly, not through the shell. Much like the `-x` option of the watch command.


-O, \--diff-output-only

:   Display only the lines with differences during `line` diff and `word` diff.


Options
-------

-A, \--aftercommand *command to execute after difference occurs*

:   Executes the specified command if the output changes. Information about changes is stored in json format in environment variable `${HWATCH_DATA}`.


-l, \--logfile *logfile*

:   Output the command execution result and its time as a log in json. The execution results that are recorded are only those that differ from the previous execution results.


-s, \--shell *shell command*

:   shell to use at runtime. can  also insert the command to the location specified by {COMMAND}.


-n, \--interval *seconds*

:   Specify update interval. The command will not allow quicker than **0.001** second interval, in which the smaller values are converted. Both '.' and ',' work for any locales.


\--tab-size *num*

:   Specifying tab display size. default 4 char.


-d, \--differences *[none, watch, line, word]*

:   set diff mode. highlight changes between updates. If only `-d` is specified, it will be a watch diff.

      *plane* ... Do not show diff (default).

      *watch* ... Diff like watch command. Specifying the *-d* option applies this mode.

      *line*  ... Can be done diff in line units.

      *word*  ... Can be done diff in line word units.



-o, \--output *[output, stdout, stderr]*

:   set output mode. If you specify the output mode, the history pane will also display only the history where the specified output mode has changed.


Configuration
-------

If you always want to use some command-line options, you can set them in the
`HWATCH` environment variable. For example, if you use `bash`, you can add
the following to your `.bashrc`:

```bash
export HWATCH="--no-title --color --no-help-banner"
```


KEYBINDS
========

**hwatch** uses *Keybind* for operations on the command execution screen.

h

:   Show help message. Press the *h* key again to return to the previous screen.


q

:   Exit hwatch.


c

:   Interprets ANSI colors and style sequences and displays them in color. This is the same as the *-c(--color)* option. Press the *c* key again to return to the original.


n

:   Outputs the line number at the beginning of the line.


r

:   Displays the output of the watch pane in reverse order.


d

:   Highlight changes between updates. The diff specified by this flag is similar to the *watch* command. This is the same as the *-d(--differences)* option. You can switch the diff mode by pressing the *d* key. The *d* key toggles these in order. Use the *0*, *1*, and *2* keys to switch directly to each mode.

      *plane* ... Do not show diff (default).

      *watch* ... Diff like watch command. Specifying the *-d* option applies this mode.

      *line*  ... Can be done diff in line units.

      *word*  ... Can be done diff in line word units.


o

:   Switch output mode at stdout, stderr, and output. If you specify the output mode, the history pane will also display only the history where the specified output mode has changed.


O

:   Display only the lines with differences during `line` diff and `word` diff.


t

:   Switch display of header and history pane.


Backspace

:   Switch display of history pane.


m

:   Switch Mouse wheel support mode. With this option, copying text with your terminal may be harder. Try holding the Shift key.


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

\+

:   Increase interval by 0.5 seconds.

\-

:   Decrease interval by 0.5 seconds (As long as it's positive).

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
