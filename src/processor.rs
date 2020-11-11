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
    struct Marker {
        sub: Subtitle,
        is_ad: bool,
    }
    fn is_subset(prev: &Marker, cur: &Marker) -> bool {
        let prev = &prev.sub.text[0];
        let cur = &cur.sub.text[0];

        // The previous subtitle must contain the current subtitle,
        // and the index of the current subtitle in the previous subtitle
        // should be zero, for it to be considered to be a match.
        // Additionally, the previous subtitle should be at most 2
        // characters shorter than the current subtitle.
        prev.find(cur).map_or(false, |i| i == 0)
        && prev.len().saturating_sub(cur.len()) <= 2
    }
    
    let regexes = load_regex()?;
    let mut marked_subs: Vec<Marker> = subs
        .into_iter()
        .map(|sub| {
            let is_ad = regexes.iter().any(|r| is_match(r, &sub));
            Marker { sub, is_ad }
        }).collect();
    
    let mut prv_mrk = None;
    for marker in marked_subs.iter_mut().rev() {
        prv_mrk = if marker.is_ad {
            Some(marker)
        } else if prv_mrk.as_ref().map_or(false, |p| is_subset(p, &marker)) {
            marker.is_ad = true;
            eprintln!("Matched subset (base: '{}', sub: '{}'", prv_mrk.unwrap().sub.text[0], marker.sub.text[0]);
            Some(marker)
        } else {
            None
        }
    }
    let filtered = marked_subs.into_iter()
        .filter(|m| !m.is_ad)
        .map(|m| m.sub)
        .collect();
    Ok(filtered)
}

fn is_match(regex: &Regex, subtitle: &Subtitle) -> bool {
    subtitle.text.iter().any(|line| {
        let mtch = regex.is_match(line);
        if mtch {
            eprintln!("Matched \"{}\" against /{}/", line, regex);
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
    eprintln!("{:#?}", patterns.clone().collect::<Vec<&str>>());
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
