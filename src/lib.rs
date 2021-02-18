use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// Describes the type of interval that an Event possesses. Note that 'None' is a value of this
/// enum.
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

/// The universal event struct.
#[derive(Debug, Deserialize, Serialize)]
pub struct Event {
    pub start: Option<DateTime<Local>>,
    pub interval: Interval,
    pub description: String,
}

impl Event {
    // TODO: actually use this in the implementation to keep repeats up-to-date.
    /// Cycles the start and end times of the interval. Applies only to in/definite repeating intervals.
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

    /// Returns true if the event, or the end of the interval, has yet to happen.
    pub fn is_upcoming(&self) -> bool {
        match self.start {
            Some(time) => Local::now() < time,
            _ => true, // untimed events are always considered upcoming
        }
    }
}

// Events are ordered chronologically first, with untimed events coming last, then alphabetically
// by description.
impl Ord for Event {
    fn cmp(&self, other: &Self) -> Ordering {
        // prelim sorts only chronologically, with untimed events coming last
        let prelim = match (self.start, other.start) {
            (Some(self_time), Some(other_time)) => match (&self.interval, &other.interval) {
                (Interval::None, Interval::None) => self_time.cmp(&other_time),
                (Interval::RepDefinite { end: self_end, .. }, Interval::None)
                    | (Interval::RepIndefinite(self_end), Interval::None)
                    | (Interval::Standard(self_end), Interval::None)
                    => self_end.cmp(&other_time),
                (Interval::None, Interval::RepDefinite { end: other_end, .. })
                    | (Interval::None, Interval::RepIndefinite(other_end))
                    | (Interval::None, Interval::Standard(other_end))
                    => self_time.cmp(&other_end),
                (Interval::RepDefinite { end: self_end, .. }, Interval::RepDefinite { end: other_end, .. })
                    | (Interval::RepDefinite { end: self_end, .. }, Interval::RepIndefinite(other_end))
                    | (Interval::RepDefinite { end: self_end, .. }, Interval::Standard(other_end))
                    | (Interval::RepIndefinite(self_end), Interval::RepDefinite { end: other_end, .. })
                    | (Interval::RepIndefinite(self_end), Interval::RepIndefinite(other_end))
                    | (Interval::RepIndefinite(self_end), Interval::Standard(other_end))
                    | (Interval::Standard(self_end), Interval::RepDefinite { end: other_end, .. })
                    | (Interval::Standard(self_end), Interval::RepIndefinite(other_end))
                    | (Interval::Standard(self_end), Interval::Standard(other_end))
                    => self_end.cmp(&other_end),
            },
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => Ordering::Equal,
        };
        // this match sorts additionally alphabetically by description
        match prelim {
            Ordering::Equal => self.description.cmp(&other.description),
            _ => prelim,
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
