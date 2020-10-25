use crate::srt::Subtitle;

use std::time::Duration;

use anyhow::{Context, Result};
use regex::Regex;

pub fn process(subs: Vec<Subtitle>) -> Result<Vec<Subtitle>> {
    let subs = insert_smoketest(subs);
    let subs = strip_ads(subs)?;
    Ok(generate_seqnum(subs))
}

fn insert_smoketest(mut subs: Vec<Subtitle>) -> Vec<Subtitle> {
    if let Some(first_sub) = subs.first() {
        let mut hide_at = first_sub.show_at;
        if hide_at.as_secs() >= 5 {
            hide_at = Duration::from_secs(5);
        }
        if hide_at.as_secs() > 0 {
            subs.insert(
                0,
                Subtitle {
                    sequence_number: None,
                    show_at: Duration::from_secs(0),
                    hide_at,
                    text: vec!["Subtitles loaded.".to_string()],
                },
            )
        }
    }
    subs
}

fn strip_ads(subs: Vec<Subtitle>) -> Result<Vec<Subtitle>> {
    let regexes = load_regex()?;
    Ok(subs
        .into_iter()
        .filter(|s| !regexes.iter().any(|r| is_match(r, s)))
        .collect())
}

fn is_match(regex: &Regex, subtitle: &Subtitle) -> bool {
    subtitle.text.iter().any(|line| {
        let mtch = regex.is_match(line);
        if mtch {
            println!("Matched \"{}\" against /{}/", line, regex);
        }
        mtch
    })
}

fn load_regex() -> Result<Vec<Regex>> {
    let patterns =
        std::fs::read_to_string("drop-subs.txt").context("Failed to read regex file.")?;
    let patterns = patterns
        .split('\n')
        .map(|p| p.trim_start())
        .filter(|p| !p.is_empty() && !p.starts_with('#'));
    println!("{:#?}", patterns.clone().collect::<Vec<&str>>());
    patterns
        .map(|p| Regex::new(p).context("Invalid regex."))
        .collect()
}

fn generate_seqnum(subs: Vec<Subtitle>) -> Vec<Subtitle> {
    let mut seqnum = 0;
    subs.into_iter()
        .map(|mut s| {
            seqnum += 1;
            s.sequence_number = Some(seqnum);
            s
        })
        .collect()
}
