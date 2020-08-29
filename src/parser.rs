use crate::srt::Subtitle;
use crate::error::SubvertError;

use std::time::Duration;

use nom::bytes::complete::{tag, take_while1, take_while_m_n};
use nom::character::complete::{digit1, line_ending, space1, multispace0, multispace1};
use nom::multi::{many_till, many0};
use nom::combinator::{map_res, opt};
use nom::error::{ErrorKind, ParseError};
use nom::sequence::{terminated, pair};
use nom::{branch::alt, Err, IResult, error_position};

pub struct Parser;
impl Parser {
    pub fn new() -> Self {
        Self {}
    }

    pub fn parse(&mut self, input: &str) -> Option<Vec<Subtitle>> {
        match srt_file(input) {
            Ok((_, subs)) => {
                Some(subs)
            },
            Err(err) => {
                dbg!(err);
                None
            },
        }
    }
}

fn optional_bom(input: &str) -> IResult<&str, Option<&str>> {
    opt(tag("\u{FEFF}"))(input)
}

fn srt_file(input: &str) -> IResult<&str, Vec<Subtitle>> {
    let (input, _) = optional_bom(input)?;
    let (input, subs) = all_subtitles(input)?;
    let (input, _) = end_of_file(input)?;
    Ok((input, subs))
}

fn all_subtitles(input: &str) -> IResult<&str, Vec<Subtitle>> {
    let mut parsed_subs = Vec::new();
    let mut input = input;
    loop {
        match subtitle(input) {
            Ok((rem_input, subtitle)) => {
                parsed_subs.push(subtitle);
                input = rem_input;
                let (rem_input, _) = multispace0(input)?;
                input = rem_input;
            },
            Err(err) => {
                if input.is_empty() {
                    return Ok((input, parsed_subs));
                } else {
                    return Err(err);
                }
            },
        }
    }
}

fn subtitle(input: &str) -> IResult<&str, Subtitle> {
    let (input, _) = multispace0(input)?;
    let (input, seqnum) = terminated(seq_num, multispace1)(input)?;
    let (input, (show_at, hide_at)) = terminated(show_hide, line_ending)(input)?;
    let (input, text) = sub_text(input)?;

    Ok((
        input,
        Subtitle {
            sequence_number: seqnum,
            show_at,
            hide_at,
            text,
        },
    ))
}

fn end_of_file(input: &str) -> IResult<&str, &str> {
    if input.is_empty() {
      Ok((input, input))
    } else {
      std::result::Result::Err(Err::Error(error_position!(input, ErrorKind::Eof)))
    }
}

fn sub_text(input: &str) -> IResult<&str, Vec<String>> {
    let line = terminated(take_while1(|c: char| c != '\n' && c != '\r'), alt((line_ending, end_of_file)));

    let (input, (vec, _)) = many_till(line, alt((line_ending, end_of_file)))(input)?;

    Ok((input, vec.into_iter().map(String::from).collect()))
}

fn show_hide(input: &str) -> IResult<&str, (Duration, Duration)> {
    let (input, show_at) = timestamp(input)?;
    let (input, _) = space1(input)?;
    let (input, _) = tag("-->")(input)?;
    let (input, _) = space1(input)?;
    let (input, hide_at) = timestamp(input)?;

    Ok((input, (show_at, hide_at)))
}

fn timestamp(input: &str) -> IResult<&str, Duration> {
    let take_n_digits = |n| {
        map_res(take_while_m_n(n, n, |c: char| c.is_digit(10)), |s: &str| {
            s.parse()
        })
    };

    let (input, hours): (_, u64) = take_n_digits(2)(input)?;
    let (input, _) = tag(":")(input)?;
    let (input, minutes) = take_n_digits(2)(input)?;
    let (input, _) = tag(":")(input)?;
    let (input, seconds) = take_n_digits(2)(input)?;
    let (input, _) = tag(",")(input)?;
    let (input, millis) = take_n_digits(3)(input)?;

    Ok((
        input,
        Duration::from_millis(
            millis + seconds * 1000 + minutes * 60 * 1000 + hours * 60 * 60 * 1000,
        ),
    ))
}

fn seq_num(input: &str) -> IResult<&str, usize> {
    map_res(digit1, |s: &str| s.parse())(input)
}
