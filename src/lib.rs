use chrono::{DateTime, Duration, Local, Timelike, TimeZone};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::io;

const DATE_FORMAT: &str = "%FT%R";

#[derive(Debug, Deserialize, Serialize)]
pub enum Interval {
    RepDefinite {
        occurrences: usize,
        duration: Duration,
    },
    RepIndefinite(Duration),
    Standard(Duration),
    None,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Event {
    start: Option<DateTime<Local>>,
    interval: Interval,
    pub description: String,
}

impl Event {
    /// Cycles the start and end times of the interval. Applies only to in/deifnite repeating intervals.
    pub fn advance(&mut self) {
        match self.interval {
            Interval::RepDefinite { occurrences, duration } => {
                self.start = Some(self.start.unwrap() + duration);
                self.interval = Interval::RepDefinite {
                    occurrences: occurrences - 1,
                    duration,
                };
            },
            Interval::RepIndefinite(duration) => {
                self.start = Some(self.start.unwrap() + duration);
            },
            _ => {},
        }
    }

    pub fn end_time(&self) -> Option<String> {
        match self.interval {
            Interval::RepDefinite { duration, .. }
                | Interval::RepIndefinite(duration)
                | Interval::Standard(duration)
            => Some(
                (self.start.unwrap() + duration).format(DATE_FORMAT).to_string()
            ),
            Interval::None => None,
        }
    }

    // record syntax: "[iso string]\t[description]"
    fn from_record(line: &str) -> Result<Self, Error> {
        let mut event = Event {
            start: None,
            interval: Interval::None,
            description: String::new()
        };
        let mut parts = line.split('\t');
        match parts.next() {
            Some("") => {}, // leave empty fields as assumed.
            Some(string) => {
                let mut tokens = string.split('/');
                match (tokens.next(), tokens.next(), tokens.next()) {
                    // repeating interval branch.
                    (Some(repetition), Some(start), Some(end)) => {
                        // set start date.
                        event.start = Some(Local.datetime_from_str(start, DATE_FORMAT)?);
                        // calculate duration.
                        let duration = Local.datetime_from_str(end, DATE_FORMAT)? - event.start.unwrap();
                        // calculate occurrences.
                        let occurrences: Option<usize> = match repetition.strip_prefix('R') {
                            Some("") => None, // leave occurrences as None.
                            Some(string) => {
                                match string.parse::<usize>() {
                                    Ok(num) => Some(num),
                                    Err(_) => return Err(Error::InvalidIso),
                                }
                            },
                            None => None, // leave occurrences as None.
                        };
                        if let Some(occurrences) = occurrences {
                            event.interval = Interval::RepDefinite { occurrences, duration };
                        } else {
                            event.interval = Interval::RepIndefinite(duration);
                        }
                    },
                    // non-repeating interval branch.
                    (Some(start), Some(end), None) => {
                        event.start = Some(Local.datetime_from_str(start, DATE_FORMAT)?);
                        event.interval = Interval::Standard(Local.datetime_from_str(end, DATE_FORMAT)? - event.start.unwrap());
                    },
                    // non-interval branch.
                    (Some(start), None, None) => {
                        event.start = Some(Local.datetime_from_str(start, DATE_FORMAT)?);
                    },
                    // untimed branch.
                    (None, None, None) => {},
                    _ => return Err(Error::InvalidIso),
                }
            },
            None => return Err(Error::InvalidIso),
        }
        match parts.next() {
            Some(string) => event.description.push_str(string),
            None => {}, // leave the empty string that we've already assumed.
        }
        Ok(event)
    }

    pub fn interval(&self) -> &Interval { &self.interval }

    pub fn is_repeating(&self) -> bool {
        match self.interval {
            Interval::RepDefinite{..} | Interval::RepIndefinite(_) => true,
            _ => false,
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

    pub fn iso_string(&self) -> String {
        match (self.start, &self.interval) {
            (Some(datetime), Interval::RepDefinite { occurrences, duration }) => {
                format!(
                    "R{}/{}/{}",
                    occurrences,
                    datetime.format(DATE_FORMAT).to_string(),
                    (datetime + *duration).format(DATE_FORMAT).to_string()
                )
            },
            (Some(datetime), Interval::RepIndefinite(duration)) => {
                format!(
                    "R/{}/{}",
                    datetime.format(DATE_FORMAT).to_string(),
                    (datetime + *duration).format(DATE_FORMAT).to_string()
                )
            },
            (Some(datetime), Interval::Standard(duration)) => {
                format!(
                    "{}/{}",
                    datetime.format(DATE_FORMAT).to_string(),
                    (datetime + *duration).format(DATE_FORMAT).to_string()
                )
            },
            (Some(datetime), Interval::None) => {
                datetime.format(DATE_FORMAT).to_string()
            },
            (None, _) => String::new(),
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

#[derive(Debug)]
pub enum Error {
    ChronoParse(chrono::format::ParseError),
    InvalidIso,
    Io(io::Error),
}

impl From<chrono::format::ParseError> for Error {
    fn from(error: chrono::format::ParseError) -> Self {
        Error::ChronoParse(error)
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::Io(error)
    }
}
