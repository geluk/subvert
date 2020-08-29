use std::time::Duration;

#[derive(Debug)]
pub struct Subtitle {
    pub(crate) sequence_number: usize,
    pub(crate) show_at: Duration,
    pub(crate) hide_at: Duration,
    pub(crate) text: Vec<String>,
}

