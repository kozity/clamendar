// TODO: replace start_time()'s with functionality native to main.rs
// TODO: create file if missing
// TODO: figure out text input in UI

use clamendar::{self, Event, Interval};
use crossterm::{
    event::{self, KeyCode},
    terminal,
};
use std::io;
use std::fs;
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    Terminal,
    widgets::{Block, Borders, List, ListItem, ListState, Row, Table, TableState},
};

const FILEPATH: &str = "./events.json";

enum Focus {
    Intervals,
    Timed,
    Untimed,
    None,
}

struct State {
    focus: Focus,
    intervals: TableState,
    timed: TableState,
    untimed: ListState,
}

#[derive(Debug)]
enum Error {
    Crossterm(crossterm::ErrorKind),
    Io(io::Error),
    Serde(serde_json::Error),
}

impl From<crossterm::ErrorKind> for Error {
    fn from(error: crossterm::ErrorKind) -> Self {
        Error::Crossterm(error)
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::Io(error)
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Error::Serde(error)
    }
}

fn main() -> Result<(), Error> {
    let mut events_intervals: Vec<Event> = Vec::new();
    let mut events_timed: Vec<Event> = Vec::new();
    let mut events_untimed: Vec<Event> = Vec::new();
    for event in deserialize()? {
        match (&event.start, &event.interval) {
            (Some(_), Interval::None) => events_timed.push(event),
            (Some(_), _) => events_intervals.push(event),
            (None, _) => events_untimed.push(event),
        }
    }
    events_timed.sort_unstable();
    events_intervals.sort_unstable();

    let mut state = State {
        focus: Focus::None,
        intervals: TableState::default(),
        timed: TableState::default(),
        untimed: ListState::default(),
    };
    if !events_intervals.is_empty() {
        state.intervals.select(Some(0));
    }
    if !events_timed.is_empty() {
        state.timed.select(Some(0));
    }
    if !events_untimed.is_empty() {
        state.untimed.select(Some(0));
    }

    terminal::enable_raw_mode()?;
    let mut terminal = Terminal::new(
        CrosstermBackend::new(
            io::stdout()
        )
    )?;
    terminal.clear()?;

    loop {
        terminal.draw(|term| {
            let main_rectangle = term.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Percentage(30),
                        Constraint::Percentage(70),
                    ]
                    .as_ref()
                )
                .split(main_rectangle);
            let chunks_bottom = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [
                        Constraint::Percentage(60),
                        Constraint::Percentage(40),
                    ]
                )
                .split(chunks[1]);

            // Lay out intervals in the top block.
            let mut table_intervals = Table::new(
                events_intervals
                    .iter()
                    .map(|event| {
                        Row::new(vec![
                            event.start_time().unwrap(),
                            event.description.clone(),
                        ])
                    })
                    .collect::<Vec<Row>>()
            )
            .header(Row::new(vec!["End", "Description"])
                .style(Style::default().add_modifier(Modifier::BOLD))
            )
            .block(Block::default()
                .borders(Borders::ALL)
                .title("Intervals")
            )
            .widths(&[
                Constraint::Percentage(20),
                Constraint::Percentage(80),
            ]);

            // Lay out timed events in the bottom left block.
            let mut table_timed = Table::new(
                events_timed
                    .iter()
                    .map(|event| {
                        Row::new(vec![
                            event.start_time().unwrap(),
                            event.description.clone(),
                        ])
                    })
                    .collect::<Vec<Row>>()
            )
            .header(Row::new(vec!["Time", "Description"])
                .style(Style::default().add_modifier(Modifier::BOLD))
            )
            .block(Block::default()
                .borders(Borders::ALL)
                .title("Timed")
            )
            .widths(&[
                Constraint::Percentage(20),
                Constraint::Percentage(80),
            ]);
            
            // Lay out untimed events in the bottom right block.
            let mut list_untimed = List::new(events_untimed
                .iter()
                .map(|event| ListItem::new(event.description.clone()))
                .collect::<Vec<ListItem>>()
            )
            .block(Block::default()
                .borders(Borders::ALL)
                .title("Untimed")
            );

            match state.focus {
                Focus::Intervals => {
                    table_intervals = table_intervals
                        .highlight_style(Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(Color::LightBlue)
                    );
                },
                Focus::Timed => {
                    table_timed = table_timed
                        .highlight_style(Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(Color::LightBlue)
                    );
                },
                Focus::Untimed => {
                    list_untimed = list_untimed
                        .highlight_style(Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(Color::LightBlue)
                    );
                },
                Focus::None => {},
            }

            term.render_stateful_widget(table_intervals, chunks[0], &mut state.intervals);
            term.render_stateful_widget(table_timed, chunks_bottom[0], &mut state.timed);
            term.render_stateful_widget(list_untimed, chunks_bottom[1], &mut state.untimed);
        })?;

        // TODO: handle event
        match event::read()? {
            event::Event::Key(keycode) => match keycode.code {
                KeyCode::Char('g') => state.focus = Focus::Intervals,
                KeyCode::Char('h') => state.focus = Focus::Timed,
                KeyCode::Char('j') => {
                    match state.focus {
                        Focus::Intervals => {
                            match state.intervals.selected() {
                                Some(selected) if selected < events_intervals.len() - 1 => state.intervals.select(Some(selected + 1)),
                                _ => {},
                            }
                        },
                        Focus::Timed => {
                            match state.timed.selected() {
                                Some(selected) if selected < events_timed.len() - 1 => state.timed.select(Some(selected + 1)),
                                _ => {},
                            }
                        },
                        Focus::Untimed => {
                            match state.untimed.selected() {
                                Some(selected) if selected < events_untimed.len() - 1 => state.untimed.select(Some(selected + 1)),
                                _ => {},
                            }
                        },
                        Focus::None => {},
                    }
                },
                KeyCode::Char('k') => {
                    match state.focus {
                        Focus::Intervals => {
                            match state.intervals.selected() {
                                Some(selected) if selected > 0 => state.intervals.select(Some(selected - 1)),
                                _ => {},
                            }
                        },
                        Focus::Timed => {
                            match state.timed.selected() {
                                Some(selected) if selected > 0 => state.timed.select(Some(selected - 1)),
                                _ => {},
                            }
                        },
                        Focus::Untimed => {
                            match state.untimed.selected() {
                                Some(selected) if selected > 0 => state.untimed.select(Some(selected - 1)),
                                _ => {},
                            }
                        },
                        Focus::None => {},
                    }
                },
                KeyCode::Char('l') => state.focus = Focus::Untimed,
                KeyCode::Char('q') => break,
                KeyCode::Esc => state.focus = Focus::None,
                _ => {},
            },
            _ => {},
        }
    }

    terminal::disable_raw_mode()?;
    events_timed.append(&mut events_intervals);
    events_timed.append(&mut events_untimed);
    serialize(events_timed)
}

fn deserialize() -> Result<Vec<Event>, Error> {
    let file = fs::read_to_string(FILEPATH)?;
    Ok(serde_json::from_str(&file)?)
}

fn serialize(events: Vec<Event>) -> Result<(), Error> {
    Ok(fs::write(FILEPATH, &serde_json::to_vec(&events)?)?)
}
