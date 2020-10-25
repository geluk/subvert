use crate::error::SubvertError;
use crate::srt::Subtitle;

use std::time::Duration;

use anyhow::Context;
use nom::bytes::complete::{tag, take_while1, take_while_m_n};
use nom::character::complete::{digit1, line_ending, multispace0, multispace1, space0, space1};
use nom::combinator::{map_res, opt};
use nom::error::{convert_error, ErrorKind, VerboseError};
use nom::multi::many_till;
use nom::sequence::terminated;
use nom::{branch::alt, error_position, Err, IResult};

pub struct Parser;
impl Parser {
    pub fn new() -> Self {
        Self {}
    }

    pub fn parse(&mut self, input: &str) -> Result<Vec<Subtitle>, anyhow::Error> {
        match srt_file(input) {
            Ok((_, subs)) => Ok(subs),
            Err(Err::Error(err)) | Err(Err::Failure(err)) => {
                let conv = convert_error(input, err);
                Err(SubvertError::ParseError(conv)).context("Failed to parse SRT file")
            }
            Err(Err::Incomplete(_)) => {
                unreachable!("Incomplete data received by non-streaming parser.")
            }
        }
    }
}

fn optional_bom(input: &str) -> IResult<&str, Option<&str>, VerboseError<&str>> {
    opt(tag("\u{FEFF}"))(input)
}

fn srt_file(input: &str) -> IResult<&str, Vec<Subtitle>, VerboseError<&str>> {
    let (input, _) = optional_bom(input)?;
    let (input, mut subs) = all_subtitles(input)?;
    let (input, _) = end_of_file(input)?;
    subs.sort_by_key(|s| s.show_at);
    Ok((input, subs))
}

fn all_subtitles(input: &str) -> IResult<&str, Vec<Subtitle>, VerboseError<&str>> {
    let mut parsed_subs = Vec::new();
    let mut input = input;
    loop {
        match subtitle(input) {
            Ok((rem_input, subtitle)) => {
                parsed_subs.push(subtitle);
                input = rem_input;
                let (rem_input, _) = multispace0(input)?;
                input = rem_input;
            }
            Err(err) => {
                if input.is_empty() {
                    return Ok((input, parsed_subs));
                } else {
                    return Err(err);
                }
            }
        }
    }
}

fn subtitle(input: &str) -> IResult<&str, Subtitle, VerboseError<&str>> {
    let (input, _) = multispace0(input)?;
    let (input, _) = terminated(seq_num, multispace1)(input)?;
    let (input, (show_at, hide_at)) = terminated(show_hide, line_ending)(input)?;
    let (input, text) = sub_text(input)?;

    Ok((
        input,
        Subtitle {
            sequence_number: None,
            show_at,
            hide_at,
            text,
        },
    ))
}

fn end_of_file(input: &str) -> IResult<&str, &str, VerboseError<&str>> {
    if input.is_empty() {
        Ok((input, input))
    } else {
        std::result::Result::Err(Err::Error(error_position!(input, ErrorKind::Eof)))
    }
}

fn sub_text(input: &str) -> IResult<&str, Vec<String>, VerboseError<&str>> {
    let line = terminated(
        take_while1(|c: char| c != '\n' && c != '\r'),
        alt((line_ending, end_of_file)),
    );

    let (input, (vec, _)) = many_till(line, alt((line_ending, end_of_file)))(input)?;

    Ok((input, vec.into_iter().map(String::from).collect()))
}

fn show_hide(input: &str) -> IResult<&str, (Duration, Duration), VerboseError<&str>> {
    let (input, show_at) = timestamp(input)?;
    let (input, _) = space1(input)?;
    let (input, _) = tag("-->")(input)?;
    let (input, _) = space1(input)?;
    let (input, hide_at) = timestamp(input)?;
    let (input, _) = space0(input)?;

    Ok((input, (show_at, hide_at)))
}

fn timestamp(input: &str) -> IResult<&str, Duration, VerboseError<&str>> {
    const MILLIS_MIN: usize = 0;
    const MILLIS_MAX: usize = 3;
    let take_millis = || {
        map_res(
            take_while_m_n(MILLIS_MIN, MILLIS_MAX, |c: char| c.is_digit(10)),
            move |s: &str| {
                if s.len() < MILLIS_MAX {
                    // Sometimes, a milliseconds value like `,2` may be encountered.
                    // This is not valid SRT, but we must be able to handle it anyway.
                    // We choose to interpret this as `,200`. In other words, we right-pad
                    // every string until it reaches a length of 3 characters.
                    let millis = format!("{:0<3}", s);
                    millis.parse()
                } else {
                    s.parse()
                }
            },
        )
    };

    const HMS_MIN: usize = 0;
    const HMS_MAX: usize = 2;
    let take_hms = || {
        map_res(
            take_while_m_n(HMS_MIN, HMS_MAX, |c: char| c.is_digit(10)),
            |s: &str| {
                if s.len() < HMS_MAX {
                    // Unlike in the previous situation, here we left-pad the value instead,
                    // because it makes more sense to treat 1:13:45 as 01:13:45 than as 10:13:45.
                    let millis = format!("{:0>2}", s);
                    millis.parse()
                } else {
                    s.parse()
                }
            },
        )
    };

    let (input, hours): (_, u64) = take_hms()(input)?;
    let (input, _) = tag(":")(input)?;
    let (input, minutes) = take_hms()(input)?;
    let (input, _) = tag(":")(input)?;
    let (input, seconds) = take_hms()(input)?;
    let (input, _) = tag(",")(input)?;
    let (input, millis): (_, u64) = take_millis()(input)?;

    Ok((
        input,
        Duration::from_millis(
            millis + seconds * 1000 + minutes * 60 * 1000 + hours * 60 * 60 * 1000,
        ),
    ))
}

fn seq_num(input: &str) -> IResult<&str, usize, VerboseError<&str>> {
    map_res(digit1, |s: &str| s.parse())(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_write_ts {
        ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (input, expected) = $value;

                let (_, duration) = timestamp(input).unwrap();

                assert_eq!(duration.as_millis(), expected);
            }
        )*
        }
    }

    test_write_ts! {
        test_write_ts_0: ("00:00:01,200", 1200),
        test_write_ts_1: ("00:00:01,2", 1200),
        test_write_ts_2: ("00:00:01,002", 1002),
        test_write_ts_3: ("00:00:01,02", 1020),
        test_write_ts_4: ("00:00:01,", 1000),
        test_write_ts_5: ("1:1:1,200", 3661200),
        test_write_ts_6: ("01:01:01,200", 3661200),
    }
}
