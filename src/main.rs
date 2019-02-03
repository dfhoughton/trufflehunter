#[macro_use]
extern crate clap;
use clap::{App, ArgMatches};
use std::path::Path;
use std::process;
extern crate two_timer;
use two_timer::parse;
extern crate larry;
use larry::Larry;
extern crate regex;
use regex::Regex;
extern crate trufflehunter;
use trufflehunter::{fetch_lines, Problem, DEFAULT_FORMAT};

fn main() {
    let options = app().get_matches();
    if options.is_present("long_help") {
        app().print_help().ok();
        println!("\n\n{}", long_help());
        process::exit(0)
    }
    if let Some(file_name) = options.value_of("LOG") {
        if let Some(values) = options.values_of("WHEN") {
            let expr = values.collect::<Vec<&str>>().join(" ");
            match parse(&expr, None) {
                Ok((d1, d2, _)) => {
                    match Larry::new(Path::new(file_name)) {
                        Ok(larry) => {
                            let pat = options.value_of("format").unwrap_or(DEFAULT_FORMAT);
                            match Regex::new(pat) {
                                Err(error) => usage(
                                    &format!("problem with format \"{}\": {}", pat, error),
                                    options,
                                ),
                                Ok(rx) => {
                                    let start_offset = if let Some(s) = options.value_of("start") {
                                        match s.parse::<usize>() {
                                            Err(_) => {
                                                return usage(
                                                    &format!(
                                                    "cannot parse --start-line {} as a line number",
                                                    s
                                                ),
                                                    options,
                                                );
                                            }
                                            Ok(v) => {
                                                if v == 0 {
                                                    return usage(
                                                        "--start-line must be greater than 0",
                                                        options,
                                                    );
                                                }
                                                Some(v)
                                            }
                                        }
                                    } else {
                                        None
                                    };
                                    let end_offset = if let Some(e) = options.value_of("end") {
                                        match e.parse::<usize>() {
                                            Err(_) => {
                                                return usage(&format!("cannot parse --end-line {} as a line number", e), options);
                                            }
                                            Ok(v) => {
                                                if v == 0 {
                                                    return usage(
                                                        "--end-line must be greater than 0",
                                                        options,
                                                    );
                                                }
                                                Some(v)
                                            }
                                        }
                                    } else {
                                        None
                                    };
                                    if start_offset.is_some()
                                        && end_offset.is_some()
                                        && start_offset.unwrap() > end_offset.unwrap()
                                    {
                                        let start_offset = start_offset.unwrap();
                                        let end_offset = end_offset.unwrap();
                                        return usage(
                                            &format!(
                                                "--start-line {} is greater than --end-line {}",
                                                start_offset, end_offset
                                            ),
                                            options,
                                        );
                                    }
                                    if options.is_present("verbose") {
                                        println!("searching for events in the range {} - {}", d1, d2);
                                    }
                                    match fetch_lines(larry, d1, d2, start_offset, end_offset, rx) {
                                    Err(p) => match p {
                                        Problem::NoTimestamps => eprintln!("no timestamps found"),
                                        Problem::LogAfter => eprintln!("events in log are after period sought"),
                                        Problem::LogBefore => eprintln!("events in log are before period sought"),
                                        Problem::MisorderedTimestamps(i1, t1, l1, i2, t2, l2) => {
                                            eprintln!("the timestamp on line {}, {}, is misordered relative to that on line {}, {}\nline {}: {}line {}: {}", i1, t1, i2, t2, i1, l1, i2, l2)
                                        }
                                        Problem::NormallyUnreachable => unreachable!()
                                    },
                                    Ok((offset, lines)) => {
                                        if options.is_present("verbose") {
                                            if lines.len() == 0 {
                                                println!("no events found");
                                            } else {
                                                println!("lines {} - {}", offset, offset + lines.len() - 1);
                                            }
                                        }
                                        for line in lines {
                                            println!("{}", line);
                                        }
                                    }
                                }
                                }
                            }
                        }
                        Err(e) => {
                            usage(&format!("problem with file {}: {}", file_name, e), options)
                        }
                    }
                }
                Err(e) => usage(
                    &format!("problem with time \"{}\": {}", expr, e.msg()),
                    options,
                ),
            }
        } else {
            usage("no time expression provided", options);
        }
    } else {
        usage("no log file provided", options);
    }
}

fn app<'a>() -> App<'a, 'a> {
    clap_app!(
        hun =>
        (version: crate_version!())
        (author: crate_authors!("\n"))
        (about: crate_description!())
        (@arg LOG: "The log file to search in")
        (@arg WHEN: ... "The period of time to search for events in")
        (@arg format: -f --format [rx] +takes_value "The time stamp format")
        (@arg long_help: --("long-help") "Long help information explaining formats and time expressions")
        (@arg verbose: --("verbose") "Provide the precise time range and line numbers")
        (@arg start: -s --("start-line") [n] +takes_value "The first line to search from")
        (@arg end: -e --("end-line") [n] +takes_value "The last line to search to")
    )
}

fn usage<'a>(msg: &str, matches: ArgMatches<'a>) {
    println!("ERROR: {}\n\n{}", msg, matches.usage());
    process::exit(1)
}

fn long_help() -> String {
    String::from(
        r#"
Trufflehunter (hun) is a tool for quickly extracting lines from a log file
within a specified time range. Its required arguments are a log file name and
a time expression.

Examples
========

    > hun log.txt from 3 am today until 3:15
    2019-01-12 3:00:01 -- tomorrow and tomorrow and tomorrow
    sometimes there's garbage between timestamps
    2019-01-12 3:05:13 -- creeps in this petty pace from day to day
    2019-01-12 3:06:00 -- to the last syllable of recorded time
    [exit stage left pursued by bear]
    2019-01-12 3:10:23 -- and all our yesterdays have lighted fools
    2019-01-12 3:14:59 -- the way to dusty death

    > hun --verbose log.txt from 3 am today until 3:06
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

    "#,
    ) + DEFAULT_FORMAT
        + r#"

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
"#
}
