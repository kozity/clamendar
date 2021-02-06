use chrono::{DateTime, Local, Timelike};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

const DATE_FORMAT: &str = "%FT%R";

#[derive(Debug, Deserialize, Serialize)]
pub enum Interval {
    RepDefinite {
        occurrences: usize,
        end: DateTime<Local>,
    },
    RepIndefinite(DateTime<Local>),
    Standard(DateTime<Local>),
    None,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Event {
    pub start: Option<DateTime<Local>>,
    pub interval: Interval,
    pub description: String,
}

impl Event {
    /// Cycles the start and end times of the interval. Applies only to in/deifnite repeating intervals.
    pub fn advance(&mut self) {
        match self.interval {
            Interval::RepDefinite { occurrences, end } => {
                let duration = end - self.start.unwrap();
                self.start = Some(end);
                self.interval = Interval::RepDefinite {
                    occurrences: occurrences - 1,
                    end: end + duration,
                };
            },
            Interval::RepIndefinite(end) => {
                let start = self.start.unwrap();
                self.start = Some(end);
                self.interval = Interval::RepIndefinite(end + (end - start));
            },
            _ => {},
        }
    }

    pub fn end_time(&self) -> Option<String> {
        match self.interval {
            Interval::RepDefinite { end, .. }
                | Interval::RepIndefinite(end)
                | Interval::Standard(end)
            => Some(end.format(DATE_FORMAT).to_string()),
            Interval::None => None,
        }
    }

    pub fn is_upcoming(&self) -> bool {
        match self.start {
            Some(time) => Local::now() < time,
            _ => true, // untimed events are always relevant.
        }
    }

    pub fn start_time(&self) -> Option<String> {
        match self.start {
            Some(datetime) => {
                let time = datetime.time();
                if time.hour() == 0 && time.minute() == 0 {
                    Some(datetime.format("%F").to_string())
                } else {
                    Some(datetime.format(DATE_FORMAT).to_string())
                }
            },
            None => None,
        }
    }
}

impl Ord for Event {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.start, other.start) {
            (Some(self_time), Some(other_time)) => self_time.cmp(&other_time),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => Ordering::Equal,
        }
    }
}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq<Self> for Event {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for Event {}
