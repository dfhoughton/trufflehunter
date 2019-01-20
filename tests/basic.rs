// some sanity tests
extern crate trufflehunter;
use std::fs;
use trufflehunter::{fetch_lines, Problem, DEFAULT_FORMAT};
extern crate chrono;
use chrono::{Duration, NaiveDate, NaiveDateTime, Timelike};
extern crate regex;
use regex::Regex;
#[macro_use]
extern crate lazy_static;
use std::path::Path;
extern crate larry;
use larry::Larry;

lazy_static! {
    static ref DATE: Regex = Regex::new(DEFAULT_FORMAT).unwrap();
    static ref LOG_LINE: Regex = Regex::new(r"(?m)^(.*?)\s+(\d+) (\d+)\s*$").unwrap();
}

// parse a date
fn date(s: &str) -> NaiveDateTime {
    let caps = DATE.captures(s).unwrap();
    let year = caps["year"].parse::<i32>().unwrap();
    let month = caps["month"].parse::<u32>().unwrap();
    let day = caps["day"].parse::<u32>().unwrap();
    let hour = caps["hour"].parse::<u32>().unwrap();
    let minute = caps["minute"].parse::<u32>().unwrap();
    let second = caps["second"].parse::<u32>().unwrap();
    NaiveDate::from_ymd(year, month, day).and_hms(hour, minute, second)
}

#[test]
fn simple_log() {
    let name = "simple_log.log";
    fs::write(
        name,
        r#"
2000-1-1 1:00:00 not it
2000-1-2 1:00:00 not it
2000-1-2 2:00:00 not it
2000-1-3 1:00:00 not it
2000-1-3 2:00:00 not it
2000-1-3 3:00:00 what we're looking for
2000-1-3 4:00:00 not it
2000-1-3 5:00:00 not it
2000-1-4 1:00:00 not it
2000-1-4 2:00:00 not it
2000-1-4 3:00:00 not it
"#,
    )
    .expect("could not write file");
    let larry = Larry::new(&Path::new(name)).expect("could not make larry");
    match fetch_lines(
        larry,
        date("2000-1-3 3:00:00"),
        date("2000-1-3 3:00:01"),
        None,
        None,
        DATE.clone(),
    ) {
        Err(e) => assert!(false, format!("error: {:?}", e)),
        Ok((offset, lines)) => {
            assert_eq!(6, offset);
            assert!(lines[0].contains("what we're looking for"));
        }
    }
    fs::remove_file(name).expect("could not delete file");
}

#[test]
fn log_with_garbage() {
    let name = "log_with_garbage.log";
    fs::write(
        name,
        r#"
initial garbage line
2000-1-1 1:00:00 not it
more garbage
2000-1-2 1:00:00 not it
more garbage
more garbage
more garbage
2000-1-2 2:00:00 not it
more garbage
2000-1-3 1:00:00 not it
2000-1-3 2:00:00 not it
more garbage
more garbage
more garbage
more garbage
2000-1-3 3:00:00 what we're looking for
we should also get this line
and this line
2000-1-3 4:00:00 not it
more garbage
more garbage
more garbage
2000-1-3 5:00:00 not it
more garbage
2000-1-4 1:00:00 not it
more garbage
more garbage
more garbage
2000-1-4 2:00:00 not it
more garbage
2000-1-4 3:00:00 not it
more garbage
more garbage
more garbage
"#,
    )
    .expect("could not write file");
    let larry = Larry::new(&Path::new(name)).expect("could not make larry");
    match fetch_lines(
        larry,
        date("2000-1-3 3:00:00"),
        date("2000-1-3 3:00:01"),
        None,
        None,
        DATE.clone(),
    ) {
        Err(e) => assert!(false, format!("error: {:?}", e)),
        Ok((offset, lines)) => {
            assert_eq!(16, offset);
            assert_eq!(3, lines.len());
            assert!(lines[0].contains("what we're looking for"));
            assert!(lines[1].contains("we should also get this line"));
            assert!(lines[2].contains("and this line"));
        }
    }
    fs::remove_file(name).expect("could not delete file");
}

fn extract_tests(text: &str) -> Vec<(NaiveDateTime, usize, usize)> {
    LOG_LINE
        .captures_iter(text)
        .map(|c| {
            (
                date(c.get(1).unwrap().as_str()),
                c.get(2).unwrap().as_str().parse::<usize>().unwrap(),
                c.get(3).unwrap().as_str().parse::<usize>().unwrap(),
            )
        })
        .collect::<Vec<_>>()
}

#[test]
fn log_with_garbage2() {
    let name = "foo.log";
    let text = r#"
initial garbage line
2000-1-1 1:00:00 2 2
more garbage
2000-1-2 1:00:00 4 4
more garbage
more garbage
more garbage
2000-1-2 2:00:00 8 2
more garbage
2000-1-3 1:00:00 10 1
2000-1-3 2:00:00 11 5
more garbage
more garbage
more garbage
more garbage
2000-1-3 3:00:00 16 3
we should also get this line
and this line
2000-1-3 4:00:00 19 4
more garbage
more garbage
more garbage
2000-1-3 5:00:00 23 2
more garbage
2000-1-4 1:00:00 25 4
more garbage
more garbage
more garbage
2000-1-4 2:00:00 29 2
more garbage
2000-1-4 3:00:00 31 4
more garbage
more garbage
more garbage
"#;
    fs::write(name, text).expect("could not write file");
    let tests = extract_tests(text);
    assert_eq!(11, tests.len());
    for (date, o, n) in tests {
        let larry = Larry::new(&Path::new(name)).expect("could not make larry");
        match fetch_lines(
            larry,
            date,
            date.with_second(59).unwrap(),
            None,
            None,
            DATE.clone(),
        ) {
            Err(e) => assert!(false, format!("error: {:?}", e)),
            Ok((offset, lines)) => {
                assert_eq!(o, offset);
                assert_eq!(n, lines.len());
            }
        }
    }
    fs::remove_file(name).expect("could not delete file");
}

#[test]
fn big_time() {
    let name = "big_time.log";
    let text = r#"
initial garbage line
2000-1-1 1:00:00 2 2
more garbage
2000-1-2 1:00:00 4 4
more garbage
more garbage
more garbage
2000-1-2 2:00:00 8 2
more garbage
2000-1-3 1:00:00 10 1
2000-1-3 2:00:00 11 5
more garbage
more garbage
more garbage
more garbage
2000-1-3 3:00:00 16 3
we should also get this line
and this line
2000-1-3 4:00:00 19 4
more garbage
more garbage
more garbage
2000-1-3 5:00:00 23 2
more garbage
2000-1-4 1:00:00 25 4
more garbage
more garbage
more garbage
2000-1-4 2:00:00 29 2
more garbage
2000-1-4 3:00:00 31 4
more garbage
more garbage
more garbage
"#;
    fs::write(name, text).expect("could not write file");
    let tests = extract_tests(text);
    assert_eq!(11, tests.len());
    for i in 0..tests.len() {
        let (d1, o, mut n) = tests[i];
        let d1 = d1 - Duration::seconds(1);
        let end;
        if let Some((d2, _, n2)) = tests.get(i + 1) {
            end = d2.with_second(59).unwrap();
            n += n2;
        } else {
            end = d1 + Duration::seconds(2);
        }
        let larry = Larry::new(&Path::new(name)).expect("could not make larry");
        match fetch_lines(larry, d1, end, None, None, DATE.clone()) {
            Err(e) => assert!(false, format!("error: {:?}", e)),
            Ok((offset, lines)) => {
                assert_eq!(o, offset);
                assert_eq!(n, lines.len());
            }
        }
    }
    fs::remove_file(name).expect("could not delete file");
}

#[test]
fn start_line_works() {
    let name = "start_line_works.log";
    fs::write(
        name,
        r#"
2000-1-3 2:00:00 not it
2000-1-3 2:01:00 what we're looking for
2000-1-3 3:00:00 not it
"#,
    )
    .expect("could not write file");
    let larry = Larry::new(&Path::new(name)).expect("could not make larry");
    match fetch_lines(
        larry,
        date("2000-1-3 2:00:00"),
        date("2000-1-3 2:30:00"),
        Some(3),
        None,
        DATE.clone(),
    ) {
        Err(e) => assert!(false, format!("error: {:?}", e)),
        Ok((_, lines)) => {
            assert_eq!(1, lines.len());
            assert!(lines[0].contains("what we're looking for"));
        }
    }
    fs::remove_file(name).expect("could not delete file");
}

#[test]
fn end_line_works() {
    let name = "end_line_works.log";
    fs::write(
        name,
        r#"
2000-1-3 2:00:00 what we're looking for
2000-1-3 2:01:00 not it
2000-1-3 3:00:00 not it
"#,
    )
    .expect("could not write file");
    let larry = Larry::new(&Path::new(name)).expect("could not make larry");
    match fetch_lines(
        larry,
        date("2000-1-3 2:00:00"),
        date("2000-1-3 2:30:00"),
        None,
        Some(2),
        DATE.clone(),
    ) {
        Err(e) => assert!(false, format!("error: {:?}", e)),
        Ok((_, lines)) => {
            assert_eq!(1, lines.len());
            assert!(lines[0].contains("what we're looking for"));
        }
    }
    fs::remove_file(name).expect("could not delete file");
}

#[test]
fn before_log() {
    let name = "before_log.log";
    fs::write(
        name,
        r#"
2000-1-3 2:00:00 not it
2000-1-3 2:01:00 not it
2000-1-3 3:00:00 not it
"#,
    )
    .expect("could not write file");
    let larry = Larry::new(&Path::new(name)).expect("could not make larry");
    match fetch_lines(
        larry,
        date("2000-1-3 1:00:00"),
        date("2000-1-3 1:30:00"),
        None,
        None,
        DATE.clone(),
    ) {
        Err(e) => match e {
            Problem::LogAfter => assert!(true),
            _ => {
                println!("{:?}", e);
                assert!(false, "wrong error")
            }
        },
        Ok(_) => assert!(false, "this was supposed to throw an error"),
    }
    fs::remove_file(name).expect("could not delete file");
}

#[test]
fn after_log() {
    let name = "after_log.log";
    fs::write(
        name,
        r#"
2000-1-3 2:00:00 not it
2000-1-3 2:01:00 not it
2000-1-3 3:00:00 not it
"#,
    )
    .expect("could not write file");
    let larry = Larry::new(&Path::new(name)).expect("could not make larry");
    match fetch_lines(
        larry,
        date("2000-1-3 4:00:00"),
        date("2000-1-3 4:30:00"),
        None,
        None,
        DATE.clone(),
    ) {
        Err(e) => match e {
            Problem::LogBefore => assert!(true),
            _ => {
                println!("{:?}", e);
                assert!(false, "wrong error")
            }
        },
        Ok(_) => assert!(false, "this was supposed to throw an error"),
    }
    fs::remove_file(name).expect("could not delete file");
}

#[test]
fn misordered() {
    let name = "misordered.log";
    fs::write(
        name,
        r#"
2000-1-3 2:01:00 not it
2000-1-3 1:00:00 not it
"#,
    )
    .expect("could not write file");
    let larry = Larry::new(&Path::new(name)).expect("could not make larry");
    match fetch_lines(
        larry,
        date("2000-1-3 1:00:00"),
        date("2000-1-3 3:00:00"),
        None,
        None,
        DATE.clone(),
    ) {
        Err(e) => match e {
            Problem::MisorderedTimestamps(..) => assert!(true),
            _ => {
                println!("{:?}", e);
                assert!(false, "wrong error")
            }
        },
        Ok((offset, lines)) => {
            println!("offset: {}\nlines: {:?}", offset, lines);
            assert!(false, "this was supposed to throw an error")
        }
    }
    fs::remove_file(name).expect("could not delete file");
}

#[test]
fn empty() {
    let name = "empty.log";
    fs::write(
        name,
        r#"
not a timestamp
also not a timestamp
"#,
    )
    .expect("could not write file");
    let larry = Larry::new(&Path::new(name)).expect("could not make larry");
    match fetch_lines(
        larry,
        date("2000-1-3 1:00:00"),
        date("2000-1-3 3:00:00"),
        None,
        None,
        DATE.clone(),
    ) {
        Err(e) => match e {
            Problem::NoTimestamps => assert!(true),
            _ => {
                println!("{:?}", e);
                assert!(false, "wrong error")
            }
        },
        Ok((offset, lines)) => {
            println!("offset: {}\nlines: {:?}", offset, lines);
            assert!(false, "this was supposed to throw an error")
        }
    }
    fs::remove_file(name).expect("could not delete file");
}
