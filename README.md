# trufflehunter
log file searching utility

```
USAGE:
    th [FLAGS] [OPTIONS] [ARGS]

FLAGS:
    -h, --help         Prints help information
        --long-help    Long help information explaining formats and time expressions
    -V, --version      Prints version information
        --verbose      Provide the precise time range and line numbers

OPTIONS:
    -e, --end-line <n>      The last line to search to
    -f, --format <rx>       The time stamp format
    -s, --start-line <n>    The first line to search from

ARGS:
    <LOG>        The log file to search in
    <WHEN>...    The period of time to search for events in
```


Trufflehunter (th) is a tool for quickly extracting lines from a log file
within a specified time range. Its required arguments are a log file name and
a time expression.

Examples
========

    > th log.txt from 3 am today until 3:15
    2019-01-12 3:00:01 -- tomorrow and tomorrow and tomorrow
    sometimes there's garbage between timestamps
    2019-01-12 3:05:13 -- creeps in this petty pace from day to day
    2019-01-12 3:06:00 -- to the last syllable of recorded time
    [exit stage left pursued by bear]
    2019-01-12 3:10:23 -- and all our yesterdays have lighted fools
    2019-01-12 3:14:59 -- the way to dusty death

    > th --verbose log.txt from 3 am today until 3:06
    lines 12345 - 12349
    2019-01-12 3:00:01 -- tomorrow and tomorrow and tomorrow
    sometimes there's garbage between timestamps
    2019-01-12 3:05:13 -- creeps in this petty pace from day to day
    2019-01-12 3:06:00 -- to the last syllable of recorded time
    [exit stage left pursued by bear]

Time Expressions
================

Trufflehunter uses the two_timer crate to parse time expressions. The
range of time expressions it will understand would be somewhat arduous to
enumerate. In general they fit one of these patterns

    <time>
    from <time a> to <time b>
    <period> around/before/after <time>

The two_timer parser is meant to emulate the semantics of English temporal
expressions. You can experiment to see what it will accept. When in doubt,
it doesn't hurt to be be specific. E.g.,

    5 minutes before and after 3:13:45 PM on June 5, 1910

Timestamp Formats
=================

A time stamp format must be a string that can compile to a regular expression
with named capturing groups for "year", "month", "day", "minute", "hour", and
"second". No named captures are required, but a format without any captures
won't do a very good job finding timestamps. Be aware that you cannot reuse a
named capture name in Rust regexes. The expression

    (?<foo>f) (?<bar>b) | (?<bar>b) (?<foo>f)

is ill-formed.

The default format is

    (?:[^\d'"`>]|^)(?P<year>[0-9]{4})\D{1,2}(?P<month>[0-9]{1,2})\D{1,2}(?P<day>[0-9]{1,2})\D{1,2}(?P<hour>[0-9]{1,2})\D{1,2}(?P<minute>[0-9]{1,2})\D{1,2}(?P<second>[0-9]{1,2})(?:[^\d'"`<]|$)

This is meant to match most log timestamps without matching quoted timestamps
in logged SQL or a data serialization language such as JSON or XML.

This is obviously quite a bit to type on the command line. Most often the
default pattern will match a log file's timestamps.

If a line contains no timestamp it will be treated as having the same timestamp
as closest line before it with a timestamp.

Start and End Lines
===================

The --start-line and --end-line options allow one to work around misordered log
lines or failures of the time format. If trufflehunter errors out because it
encounters misordered, or "misordered", lines, you may be able to work around
these rough patches by specifying a sub-range of lines to search in.

The Name
========

The truffles trufflehunter hunts are the desired periods in logs. Trufflehunter
is also the name of a talking badger in the Chronicles of Narnia.