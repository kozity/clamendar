use serde::{Serialize, Deserialize};
use std::cmp::Ordering;
use std::fs;
use std::io;

/// A struct comprising an optional ISO 8601 time format and a single general text field.
#[derive(Serialize, Deserialize, Debug)]
pub struct Task {
    pub string: Option<String>,
    pub description: String,
} // struct Task

impl Task {
    /*
    // TODO: this is gonna be real tough. Probably bring in chrono crate.
    /// Adjusts a repeating interval string so that the end time becomes the new start time.
    /// Decrements the repetition count if applicable.
    pub fn advance_repeating(&mut self) {
        if let None = self.string { return; }
        if !self.string.unwrap().starts_with('R') { return; }

        let start = self.start_time().unwrap();
        let end = self.end_time().unwrap();

        // get duration between
        
        // create next end time
        //let next = 

        // set new string
    } // advance_repeating()
    */

    /// Returns the simple end timestamp of a (repeating) interval, or just the time if the task
    /// does not involve an interval.
    pub fn end_time(&self) -> Option<String> {
        if let None = self.string { return None; }
        let string = self.string.as_ref().unwrap();
        if !string.contains('/') { return Some(string.clone()); }
        let s = if string.starts_with('R') {
            string.split('/').nth(2)
        } else {
            string.split('/').nth(1)
        };
        // TODO: see start_time() todo.
        match s {
            Some(s) => Some(String::from(s)),
            _ => None,
        }
    } // end_time()

    /// Returns the simple start timestamp of a (repeating) interval, or just the time if the task
    /// does not involve an interval.
    pub fn start_time(&self) -> Option<String> {
        if let None = self.string { return None; }
        let string = self.string.as_ref().unwrap();
        let s = if string.starts_with('R') {
            string.split('/').nth(1)
        } else {
            string.split('/').nth(0)
        };
        // TODO: potential case for better error handling/propagation here.
        match s {
            Some(s) => Some(String::from(s)),
            _ => None,
        }
    } // start_time()
} // impl Task

impl PartialEq for Task {
    fn eq(&self, other: &Self) -> bool {
        match (self.start_time(), other.start_time()) {
            (None, None) => true,
            (Some(left), Some(right)) => left == right,
            _ => false,
        }
    }
} // Task trait PartialEq

impl Eq for Task {}

impl PartialOrd<Self> for Task {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
} // Task trait PartialOrd

impl Ord for Task {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.start_time(), other.start_time()) {
            (None, None) => Ordering::Equal,
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (Some(left), Some(right)) => left.cmp(&right),
        }
    }
} // Task trait Ord

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Serde(serde_json::Error),
} // enum Error

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::Io(error)
    }
} // Error trait From<io::Error>

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Error::Serde(error)
    }
} // Error trait From<serde_json::Error>

/// Deserializes a vector of tasks from a JSON file.
pub fn deserialize(filename: &str) -> Result<Vec<Task>, Error> {
    let tasks: Vec<Task> = serde_json::from_str(&fs::read_to_string(filename)?)?;
    Ok(tasks)
} // deserialize()

/// Serializes the given vector of tasks to a JSON file.
pub fn serialize(filename: &str, tasks: Vec<Task>) -> Result<(), Error> {
    let file = fs::File::create(filename)?;
    serde_json::to_writer(file, &tasks)?;
    Ok(())
} // serialize()

#[cfg(test)]
mod tests {
    use crate::Task;
    use std::cmp::Ordering;

    macro_rules! task_a {
        () => {
            Task {
                string: Some(String::from("2021-01-01")),
                description: String::from("the first task"),
            }
        };
    }
    macro_rules! task_a_2 {
        () => {
            Task {
                string: Some(String::from("2021-01-01")),
                description: String::from("the first task's sibling"),
            }
        };
    }
    macro_rules! task_b {
        () => {
            Task {
                string: Some(String::from("R5/2021-03-03/2021-04-04")),
                description: String::from("the second task"),
            }
        };
    }
    macro_rules! task_untimed {
        () => {
            Task {
                string: None,
                description: String::from("the untimed task"),
            }
        };
    }

    #[test]
    fn serialize_basic() {
        let tasks = vec!(task_a!(), task_untimed!());
        assert_eq!(
            serde_json::to_string(&tasks).unwrap(),
            // keep updated to match task_a
            r#"[{"string":"2021-01-01","description":"the first task"},{"string":null,"description":"the untimed task"}]"#
        );
    } // serialize_basic()

    #[test]
    fn deserialize_basic() {
        let deser_tasks: Vec<Task> = serde_json::from_str(
            r#"[{"string":"2021-01-01","description":"the first task"},{"string":null,"description":"the untimed task"}]"#
        ).unwrap();
        let tasks = vec!(task_a!(), task_untimed!());
        assert_eq!(format!("{:?}", tasks), format!("{:?}", deser_tasks));
    } // deserialize_basic()

    #[test]
    fn timed_task_ordering() {
        let (task_a, task_b) = (task_a!(), task_b!());

        // informational
        assert_eq!(Ordering::Less, "2021-01-01".cmp("2021-01-02"));
        assert!(task_a < task_b);
    } // timed_task_ordering()

    #[test]
    fn timed_task_equality() {
        let (task_a, task_a_2) = (task_a!(), task_a_2!());

        assert_eq!(task_a, task_a_2);
    } // timed_task_equality()

    #[test]
    fn timed_test_inequality() {
        let (task_a, task_b) = (task_a!(), task_b!());

        assert_ne!(task_a, task_b);
    }

    #[test]
    fn timed_untimed_ordering() {
        assert!(task_a!() < task_untimed!());
    }
} // mod test
