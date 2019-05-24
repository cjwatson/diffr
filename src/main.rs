use std::io::{self, BufRead};
use termcolor::{/*Color,*/ ColorChoice, ColorSpec, StandardStream, WriteColor};
use crate::AnsiTextFormatting::*;

fn main() -> io::Result<()> {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    let mut buffer = vec![];
    let stdin = io::stdin();
    let mut handle = stdin.lock();

    loop {
        handle.read_until(b'\n', &mut buffer)?;
        if buffer.is_empty() {
            break;
        }
        let color_spec = ColorSpec::new();
        stdout.set_color(&color_spec)?;
        // dbg!(&String::from_utf8_lossy(&buffer));
        print!("{}", String::from_utf8_lossy(&buffer));
        buffer.clear();
    }

    stdout.reset()?;
    Ok(())
}

fn is_ansi_text_attribute(current: u8) -> Option<AnsiTextFormatting> {
    let result = match current {
        b'0' => Some(NormalDisplay),
        b'1' => Some(Bold),
        b'4' => Some(Underline),
        b'5' => Some(Blink),
        b'7' => Some(ReverseVideo),
        b'8' => Some(Invisible),
        _ => None,
    };
    dbg!((current, result));
    result
}

fn is_ansi_color_attribute(previous: u8, current: u8) -> Option<AnsiTextFormatting> {
    let result = match (previous, current) {
        // foreground colors
        (b'3', b'0') => Some(ForegroundBlack),
        (b'3', b'1') => Some(ForegroundRed),
        (b'3', b'2') => Some(ForegroundGreen),
        (b'3', b'3') => Some(ForegroundYellow),
        (b'3', b'4') => Some(ForegroundBlue),
        (b'3', b'5') => Some(ForegroundMagenta),
        (b'3', b'6') => Some(ForegroundCyan),
        (b'3', b'7') => Some(ForegroundWhite),
        // background colors
        (b'4', b'0') => Some(BackgroundBlack),
        (b'4', b'1') => Some(BackgroundRed),
        (b'4', b'2') => Some(BackgroundGreen),
        (b'4', b'3') => Some(BackgroundYellow),
        (b'4', b'4') => Some(BackgroundBlue),
        (b'4', b'5') => Some(BackgroundMagenta),
        (b'4', b'6') => Some(BackgroundCyan),
        (b'4', b'7') => Some(BackgroundWhite),
        _ => None,
    };
    dbg!((previous, current, result));
    result
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AnsiTextFormatting {
    // attributes
    NormalDisplay,
    Bold,
    Underline,
    Blink,
    ReverseVideo,
    Invisible,
    // foreground colors
    ForegroundBlack,
    ForegroundRed,
    ForegroundGreen,
    ForegroundYellow,
    ForegroundBlue,
    ForegroundMagenta,
    ForegroundCyan,
    ForegroundWhite,
    // background colors
    BackgroundBlack,
    BackgroundRed,
    BackgroundGreen,
    BackgroundYellow,
    BackgroundBlue,
    BackgroundMagenta,
    BackgroundCyan,
    BackgroundWhite,
}

fn parse_escape_code_prefix(bytes: &[u8]) -> Option<Vec<AnsiTextFormatting>> {
    eat_parse_escape_codes(&mut bytes.iter())
}

fn eat_parse_escape_codes(it: &mut std::slice::Iter<u8>) -> Option<Vec<AnsiTextFormatting>> {
    let mut result = vec![];
    if *it.next()? == b'\x1b' && *it.next()? == b'[' {
        let mut previous = None;
        let mut completed_sequence = None;
        loop {
            match previous {
                None => previous = Some(*it.next()?),
                Some(b'm') => {
                     if let Some(value) = completed_sequence {
                        result.push(value);
                    }
                    return Some(result)
                }
                Some(b';') => {
                    // if a sequence is completed, register it
                    if let Some(value) = completed_sequence {
                        result.push(value);
                        completed_sequence = None
                    }
                    // if multiple ';' in a row, clear
                    previous = Some(*it.next()?);
                    if previous == Some(b';') {
                        result.clear();
                    }
                }
                Some(prevbyte) => {
                    let currbyte = *it.next()?;
                    dbg!((prevbyte, currbyte));
                    match currbyte {
                        b'm' => {
                            match completed_sequence {
                                None => {
                                    let value = is_ansi_text_attribute(prevbyte)?;
                                    result.push(value);
                                    return Some(result)
                                }
                                Some(_) => return None
                            }
                        }
                        b';' => {
                            match completed_sequence {
                                None => {
                                    let value = is_ansi_text_attribute(prevbyte)?;
                                    completed_sequence = Some(value);
                                    previous = Some(b';')
                                }
                                Some(_) => return None
                            }
                        }
                        _ => {
                            match completed_sequence {
                                None => {
                                    let value = is_ansi_color_attribute(prevbyte, currbyte)?;
                                    completed_sequence = Some(value);
                                    previous = None
                                }
                                Some(_) => return None
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

#[test]
fn parse_color_escape_code_prefix_invalid() {
    assert_eq!(None, parse_escape_code_prefix(b"\x1c[32m"));
    assert_eq!(None, parse_escape_code_prefix(b"\x1b\\32m"));
    assert_eq!(None, parse_escape_code_prefix(b"\x1b[32"));
    assert_eq!(None, parse_escape_code_prefix(b"\x1b32m"));
    assert_eq!(None, parse_escape_code_prefix(b"\x1b[3233m"));
    assert_eq!(None, parse_escape_code_prefix(b"\x1b[32;433m"));
}

#[test]
fn parse_color_escape_code_prefix_simple_empty() {
    assert_eq!(Some(vec![]), parse_escape_code_prefix(b"\x1b[m"));
    assert_eq!(Some(vec![]), parse_escape_code_prefix(b"\x1b[;m"));
    assert_eq!(Some(vec![]), parse_escape_code_prefix(b"\x1b[;;m"));
}

#[test]
fn parse_color_escape_code_prefix_simple_cases() {
    assert_eq!(
        Some(vec![ForegroundGreen]),
        parse_escape_code_prefix(b"\x1b[32m"));
    assert_eq!(
        Some(vec![ForegroundRed]),
        parse_escape_code_prefix(b"\x1b[31m")
    );
    assert_eq!(
        Some(vec![Underline]),
        parse_escape_code_prefix(b"\x1b[4m")
    );
}

#[test]
fn parse_color_escape_code_prefix_with_iterator() {
    let mut it = b"\x1b[32m".iter();
    assert_eq!(
        Some(vec![ForegroundGreen]),
        eat_parse_escape_codes(&mut it)
    );
    assert_eq!(0, it.count());

    let mut it = b"\x1b[31m".iter();
    assert_eq!(
        Some(vec![ForegroundRed]),
        eat_parse_escape_codes(&mut it)
    );
    assert_eq!(0, it.count());

    let mut it = b"\x1b[4m".iter();
    assert_eq!(
        Some(vec![Underline]),
        eat_parse_escape_codes(&mut it)
    );
    assert_eq!(0, it.count());
}

#[test]
fn parse_color_escape_code_reset_on_double_59() {
    assert_eq!(
        Some(vec![ForegroundRed, BackgroundGreen]),
        parse_escape_code_prefix(b"\x1b[31;42m")
    );
    assert_eq!(
        Some(vec![BackgroundGreen]),
        parse_escape_code_prefix(b"\x1b[31;;42m")
    );
    assert_eq!(
        Some(vec![ForegroundRed, Bold, BackgroundGreen]),
        parse_escape_code_prefix(b"\x1b[31;1;42m")
    );
    assert_eq!(
        Some(vec![Invisible, Bold, BackgroundGreen]),
        parse_escape_code_prefix(b"\x1b[8;1;42m")
    );
    assert_eq!(
        Some(vec![Bold, BackgroundGreen]),
        parse_escape_code_prefix(b"\x1b[8;;1;42m")
    );
}
