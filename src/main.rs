use clamendar::{self, Event, Interval};
use std::io::{self, BufWriter, Write};
use std::fs::{self, File};
use tui::{
    backend::CrosstermBackend,
    Terminal
};
use termion::raw::IntoRawMode;

const FILEPATH: &str = "./events.json";

fn main() -> Result<(), clamendar::Error> {
    let mut events = deserialize()?;
    // TODO: define event handling
    // TODO: initialize terminal
    // TODO: define layout
    // TODO: loop
    loop {
        break;
    }

    serialize(events)
}

fn deserialize() -> Result<Vec<Event>, clamendar::Error> {
    let file = fs::read_to_string(FILENAME)?;
    serde_json::from_str(&file)
}

fn serialize(events: Vec<Event>) -> Result<(), clamendar::Error> {
    fs::write(FILEPATH, &serde_json::to_vec(&events)?)?
}
