use std::io::{self, Write};
use std::process;
use tasks;

fn main() -> Result<(), tasks::Error> {
    let filename = "/home/ty/test.json";
    let mut tasks = tasks::deserialize(filename)?;
    tasks.sort_unstable();
//    tasks_add(&mut tasks)?;
    tasks_print(&tasks);
    tasks::serialize(filename, tasks)?;
    Ok(())
} // main()

// TODO: optimize u8 vs. u32, etc.
macro_rules! OCCURRENCES_SIZE {
    () => { u32 };
}

struct TaskNew {
    repeating: bool,
    occurrences: Option<OCCURRENCES_SIZE!()>,
    start: String,
    end: Option<String>,
}

// TODO: check functionality.
fn tasks_add(tasks: &mut Vec<tasks::Task>) -> Result<(), tasks::Error> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    let mut description = String::new();
    print!("Description:\t");
    stdout.flush()?;
    stdin.read_line(&mut description)?;

    let mut string = None;

    let mut buffer = String::new();
    print!("Timed Event? (y/n)\t");
    stdout.flush()?;
    stdin.read_line(&mut buffer)?;

    if buffer.contains("y") {
        let mut config = TaskNew {
            repeating: false,
            occurrences: None,
            start: String::new(),
            end: None,
        };

        print!("(Start) Time:\t");
        stdout.flush()?;
        stdin.read_line(&mut config.start)?;
        // TODO: validate ISO 8601

        buffer.clear();
        print!("Repeating? (y/n)\t");
        stdout.flush()?;
        stdin.read_line(&mut buffer)?;
        if buffer.contains("y") { config.repeating = true; }
        
        buffer.clear();
        print!("Occurrences (0 if unbounded):\t");
        stdout.flush()?;
        stdin.read_line(&mut buffer)?;
        let occurrences = buffer.trim().parse::<OCCURRENCES_SIZE!()>();
        println!("{:?}", occurrences);
        match occurrences {
            Ok(number) => config.occurrences = Some(number),
            Err(_) => {
                eprintln!("Please provide a number.");
                process::exit(1);
            },
        }

        buffer.clear();
        print!("Interval? (y/n)\t");
        stdout.flush()?;
        stdin.read_line(&mut buffer)?;
        if buffer.contains("y") {
            buffer.clear();
            print!("End Time:\t");
            stdout.flush()?;
            stdin.read_line(&mut buffer)?;
            config.end = Some(buffer.clone());
        }

        let mut timestamp = String::new();
        if config.repeating { timestamp.push('R'); }
        if let Some(number) = config.occurrences {
            timestamp.push_str(&format!("{}", number));
        }
        timestamp.push('/');
        timestamp.push_str(&config.start);
        if let Some(string) = config.end {
            timestamp.push('/');
            // TODO: consider adding omission compatibility for ISO compliance
            timestamp.push_str(&string);
        }
        let string = Some(timestamp.trim().to_string());
    }
    tasks.push(
        tasks::Task {
            string,
            description: description.trim().to_string(),
        }
    );
    Ok(())
} // tasks_add()

fn tasks_print(tasks: &Vec<tasks::Task>) {
    for task in tasks {
        println!(
            "{}\t{}",
            match task.start_time() {
                Some(string) => string,
                _ => String::from("──────────"),
            },
            task.description
        );
    }
} // tasks_print()
