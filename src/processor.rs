use crate::srt::Subtitle;

use std::time::Duration;

use anyhow::{Context, Result};
use regex::Regex;

pub struct ProcessOpts {
    pub leader_sub: Option<String>,
}

struct Processor {
    opts: ProcessOpts,
    subs: Vec<Subtitle>,
}

impl Processor {
    fn new(opts: ProcessOpts, subs: Vec<Subtitle>) -> Processor {
        Processor {
            opts,
            subs,
        }
    }

    fn process(mut self) -> Result<Vec<Subtitle>> {
        self.insert_leader();
        self.strip_ads()?;
        self.generate_seqnum();
        Ok(self.subs)
    }

    fn insert_leader(&mut self) {
        if let Some(first_sub) = self.subs.first() {
            let mut hide_at = first_sub.show_at;
            if hide_at.as_secs() >= 5 {
                hide_at = Duration::from_secs(5);
            }
            if hide_at.as_secs() > 0 {
                self.subs.insert(
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
    }
    
    fn strip_ads(&mut self) -> Result<()> {
        struct Marker<'a> {
            sub: &'a Subtitle,
            index: usize,
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
        
        // Build a vector of subtitle markers
        let regexes = load_regex()?;
        let mut marked_subs: Vec<Marker> = self.subs
            .iter()
            .enumerate()
            .map(|(index, sub)| {
                let is_ad = regexes.iter().any(|r| is_match(r, &sub));
                Marker { sub, index, is_ad }
            }).collect();
        
        // Detect and mark 'marquee' subtitles when they are a subset of a
        // later, marked subtitle
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

        // Collect a list of indices to drop before dropping subtitles,
        // to ensure we don't modify the subs collection as we enumerate it.
        let drop_indices: Vec<usize> = marked_subs
            .iter()
            .filter(|m| m.is_ad)
            .map(|m| m.index)
            .collect();

        for i in drop_indices {
            self.subs.remove(i);
        }
        Ok(())
    }

    fn generate_seqnum(&mut self) {
        let mut seqnum = 0;
        for sub in self.subs.iter_mut() {
            seqnum += 1;
            sub.sequence_number = Some(seqnum);
        }
    }
}

pub fn process(subs: Vec<Subtitle>, opts: ProcessOpts) -> Result<Vec<Subtitle>> {
    Processor::new(opts, subs).process()
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
        include_str!("../embed/drop-subs.txt");
    let patterns = patterns
        .split('\n')
        .map(|p| p.trim_start())
        .filter(|p| !p.is_empty() && !p.starts_with('#'));
    patterns
        .map(|p| Regex::new(p).context(format!("Invalid regex: '{}'", p)))
        .collect()
}
