use std::io::{BufRead, BufReader, Seek, SeekFrom};
extern crate larry;
use larry::Larry;
extern crate chrono;
use chrono::{NaiveDate, NaiveDateTime};
extern crate regex;
use regex::Regex;

// need to make this public so it can be seen in main.rs
#[doc(hidden)]
pub const DEFAULT_FORMAT : &'static str = r#"(?:[^\d'"`>]|^)(?P<year>[0-9]{4})\D{1,2}(?P<month>[0-9]{1,2})\D{1,2}(?P<day>[0-9]{1,2})\D{1,2}(?P<hour>[0-9]{1,2})\D{1,2}(?P<minute>[0-9]{1,2})\D{1,2}(?P<second>[0-9]{1,2})(?:[^\d'"`<]|$)"#;

// need to make this public so it can be seen in main.rs
#[doc(hidden)]
#[derive(Debug)]
pub enum Problem {
    LogAfter,
    LogBefore,
    NoTimestamps,
    MisorderedTimestamps(usize, NaiveDateTime, String, usize, NaiveDateTime, String),
    NormallyUnreachable, // to mark code that should only be reachable in testing
}

// need to make this public so it can be seen in main.rs
#[doc(hidden)]
pub fn fetch_lines(
    mut larry: Larry,
    start: NaiveDateTime,
    end: NaiveDateTime,
    start_offset: Option<usize>,
    end_offset: Option<usize>,
    rx: Regex,
) -> Result<(usize, Vec<String>), Problem> {
    let i1 = if start_offset.is_some() {
        start_offset.unwrap() - 1
    } else {
        0
    };
    if let Some((mut i1, mut t1)) = get_timestamp(&mut larry, i1, &rx, true) {
        if t1 > end {
            return Err(Problem::LogAfter);
        }
        let i2 = if end_offset.is_some() {
            end_offset.unwrap() - 1
        } else {
            larry.len() - 1
        };
        if let Some((mut i2, mut t2)) = get_timestamp(&mut larry, i2, &rx, false) {
            if t2 < start {
                return Err(Problem::LogBefore);
            }
            if t2 < t1 {
                return Err(Problem::MisorderedTimestamps(
                    i1,
                    t1,
                    larry.get(i1).unwrap(),
                    i2,
                    t2,
                    larry.get(i2).unwrap(),
                ));
            }
            if t1 >= start {
                show_from(larry, i1, end, rx, end_offset)
            } else {
                // find first line in range via binary search
                loop {
                    if i2 - i1 < 10 {
                        // search linearly
                        let mut i = i1 + 1;
                        while i <= i2 {
                            let (i3, t3) = get_timestamp(&mut larry, i, &rx, true).unwrap();
                            if t3 < t1 {
                                return Err(Problem::MisorderedTimestamps(
                                    i1,
                                    t1,
                                    larry.get(i1).unwrap(),
                                    i3,
                                    t3,
                                    larry.get(i3).unwrap(),
                                ));
                            }
                            if t3 > t2 {
                                return Err(Problem::MisorderedTimestamps(
                                    i3,
                                    t3,
                                    larry.get(i3).unwrap(),
                                    i2,
                                    t2,
                                    larry.get(i2).unwrap(),
                                ));
                            }
                            if t3 >= start {
                                return show_from(larry, i3, end, rx, end_offset);
                            }
                            i = i3 + 1;
                        }
                        unreachable!();
                    }
                    let i = estimate_index(&start, i1, &t1, i2, &t2);
                    let (i3, t3) = get_timestamp(&mut larry, i, &rx, true).unwrap();
                    let (i3, t3) = if i3 == i2 {
                        get_timestamp(&mut larry, i, &rx, false).unwrap()
                    } else {
                        (i3, t3)
                    };
                    if t3 < t1 {
                        return Err(Problem::MisorderedTimestamps(
                            i1,
                            t1,
                            larry.get(i1).unwrap(),
                            i3,
                            t3,
                            larry.get(i3).unwrap(),
                        ));
                    }
                    if t3 > t2 {
                        return Err(Problem::MisorderedTimestamps(
                            i3,
                            t3,
                            larry.get(i3).unwrap(),
                            i2,
                            t2,
                            larry.get(i2).unwrap(),
                        ));
                    }
                    if t3 == start {
                        return show_from(larry, i3, end, rx, end_offset);
                    } else if t3 < start {
                        i1 = i3;
                        t1 = t3;
                    } else {
                        i2 = i3;
                        t2 = t3;
                    }
                }
            }
        } else {
            Err(Problem::NormallyUnreachable)
        }
    } else {
        Err(Problem::NoTimestamps)
    }
}

// show the lines after start index i up to a timestamp at or after end
fn show_from(
    mut larry: Larry,
    i: usize,
    end: NaiveDateTime,
    rx: Regex,
    end_offset: Option<usize>,
) -> Result<(usize, Vec<String>), Problem> {
    let offset = larry.offset(i).unwrap();
    let mut end_offset = if let Some(o) = end_offset {
        o
    } else {
        larry.len()
    };
    end_offset -= i;
    larry.file.seek(SeekFrom::Start(offset)).ok();
    let reader = BufReader::new(larry.file);
    let mut vec = vec![];
    for (i, line) in reader.lines().enumerate() {
        if i == end_offset {
            break;
        }
        let s = line.unwrap();
        if let Some(nd) = timestamp(s.as_ref(), &rx) {
            if nd >= end {
                break;
            }
        }
        vec.push(s);
    }
    Ok((i, vec))
}

// estimate the index of time t given the indices of times t1 and t2
fn estimate_index(
    t: &NaiveDateTime,
    i1: usize,
    t1: &NaiveDateTime,
    i2: usize,
    t2: &NaiveDateTime,
) -> usize {
    if t <= t1 {
        // at this point t cannot be after t2
        i1
    } else {
        let numerator = t.timestamp() - t1.timestamp();
        let denominator = t2.timestamp() - t1.timestamp();
        let f = numerator as f64 / denominator as f64;
        let n = (i2 + 1 - i1) as f64;
        let estimate = (n * f).round() as usize;
        // we know it is not either end point
        if estimate == i1 {
            i1 + 1
        } else if estimate == i2 {
            i2 - 1
        } else {
            estimate
        }
    }
}

fn get_timestamp(
    larry: &mut Larry,
    i: usize,
    time_format: &Regex,
    down: bool,
) -> Option<(usize, NaiveDateTime)> {
    let mut i = i;
    loop {
        match larry.get(i) {
            Ok(s) => {
                if let Some(nd) = timestamp(s.as_ref(), time_format) {
                    return Some((i, nd));
                }
                if down {
                    i += 1
                } else if i == 0 {
                    return None;
                } else {
                    i -= 1
                }
            }
            Err(_) => return None,
        }
    }
}

fn timestamp(line: &str, time_format: &Regex) -> Option<NaiveDateTime> {
    if let Some(captures) = time_format.captures(line) {
        let mut y = 0;
        let m;
        let d;
        let h;
        let mn;
        let s;
        if let Some(year) = captures.name("year") {
            if let Ok(year) = year.as_str().parse::<i32>() {
                y = year;
            }
        } else {
            return None;
        }
        if let Some(month) = captures.name("month") {
            if let Ok(month) = month.as_str().parse::<u32>() {
                m = month;
            } else {
                return None;
            }
        } else {
            return None;
        }
        if let Some(day) = captures.name("day") {
            if let Ok(day) = day.as_str().parse::<u32>() {
                d = day;
            } else {
                return None;
            }
        } else {
            return None;
        }
        if let Some(hour) = captures.name("hour") {
            if let Ok(hour) = hour.as_str().parse::<u32>() {
                h = hour;
            } else {
                return None;
            }
        } else {
            return None;
        }
        if let Some(minute) = captures.name("minute") {
            if let Ok(minute) = minute.as_str().parse::<u32>() {
                mn = minute;
            } else {
                return None;
            }
        } else {
            return None;
        }
        if let Some(second) = captures.name("second") {
            if let Ok(second) = second.as_str().parse::<u32>() {
                s = second;
            } else {
                return None;
            }
        } else {
            return None;
        }
        if let Some(nd) = NaiveDate::from_ymd_opt(y, m, d) {
            nd.and_hms_opt(h, mn, s)
        } else {
            None
        }
    } else {
        None
    }
}
