//use std::io;
//use std::process;
use clamendar::{self, Event, Interval};

static FILEPATH: &'static str = "/home/ty/clamendar.txt";

fn main() -> Result<(), clamendar::Error> {
    let mut events = clamendar::deserialize(FILEPATH)?;
    events.sort_unstable();
    print_events(&events);
    clamendar::serialize(FILEPATH, events)?;
    Ok(())
}

fn print_events(events: &Vec<Event>) {
    for event in events {
        match event.interval() {
            Interval::RepDefinite { occurrences, .. } => {
                if occurrences > &9 { print!("R0"); } else { print!("R{}", occurrences); }
            },
            Interval::RepIndefinite(_) => print!("R "),
            _ => print!("  "),
        }
        print!(" ");
        match event.start_time() {
            Some(string) => print!("{}", string),
            _ => print!("----------"),
        }
        print!("\t{}\n", event.description);
    }
}
