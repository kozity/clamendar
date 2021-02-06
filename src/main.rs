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
    InputAdd,
    Intervals,
    Timed,
    Untimed,
    None,
}

struct State {
    buffer: String,
    focus: Focus,
    intervals: Vec<Event>,
    intervals_offset: usize,
    intervals_state: TableState,
    timed: Vec<Event>,
    timed_offset: usize,
    timed_state: TableState,
    untimed: Vec<Event>,
    untimed_offset: usize,
    untimed_state: ListState,
}

impl State {
    fn focus(&mut self, target: Focus) {
        match self.focus {
            Focus::Intervals => self.intervals_state.select(None),
            Focus::Timed => self.timed_state.select(None),
            Focus::Untimed => self.untimed_state.select(None),
            _ => {},
        }
        match target {
            Focus::Intervals =>
                if !self.intervals.is_empty() { self.intervals_state.select(Some(self.intervals_offset)) },
            Focus::Timed =>
                if !self.timed.is_empty() { self.timed_state.select(Some(self.timed_offset)) },
            Focus::Untimed =>
                if !self.untimed.is_empty() { self.untimed_state.select(Some(self.untimed_offset)) },
            _ => {},
        }
        self.focus = target;
    }

    fn scroll_down(&mut self) {
        match s.focus {
            Focus::Intervals => match s.intervals_state.selected() {
                Some(selected) if selected < $v.len() - 1 => {
                    s.intervals_offset = selected + 1;
                    s.intervals_state.select(Some(s.intervals_offset));
                },
                _ => {},
            },
            Focus::Timed => match s.timed_state.selected() {
                Some(selected) if selected < $v.len() - 1 => {
                    s.timed_offset = selected + 1;
                    s.timed_state.select(Some(s.timed_offset));
                },
                _ => {},
            },
            Focus::Untimed => match s.untimed_state.selected() {
                Some(selected) if selected < $v.len() - 1 => {
                    s.untimed_offset = selected + 1;
                    s.untimed_state.select(Some(s.untimed_offset));
                },
                _ => {},
            },
        }
    }

    fn scroll_up(&mut self)
        match s.focus {
            Focus::Intervals => match s.intervals_state.selected() {
                Some(selected) if selected > 0 => {
                    s.intervals_offset = selected - 1;
                    s.intervals_state.select(Some(s.intervals_offset));
                },
                _ => {},
            },
            Focus::Timed => match s.timed_state.selected() {
                Some(selected) if selected > 0 => {
                    s.timed_offset = selected - 1;
                    s.timed_state.select(Some(s.timed_offset));
                },
                _ => {},
            },
            Focus::Untimed => match s.untimed_state.selected() {
                Some(selected) if selected > 0 => {
                    s.untimed_offset = selected - 1;
                    s.untimed_state.select(Some(s.untimed_offset));
                },
                _ => {},
            },
        }
    }
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
    let mut s = State {
        buffer: String::new(),
        focus: Focus::None,
        intervals: Vec::new(),
        intervals_offset: 0,
        intervals_state: TableState::default(),
        timed: Vec::new(),
        timed_offset: 0,
        timed_state: TableState::default(),
        untimed: Vec::new(),
        untimed_offset: 0,
        untimed_state: ListState::default(),
    };
    for event in deserialize()? {
        match (&event.start, &event.interval) {
            (Some(_), Interval::None) => s.timed.push(event),
            (Some(_), _) => s.intervals.push(event),
            (None, _) => s.untimed.push(event),
        }
    }
    s.intervals.sort_unstable();
    s.timed.sort_unstable();

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
                        Constraint::Min(0),
                        Constraint::Length(0),
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
            let table_intervals = Table::new(
                s.intervals
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
            ])
            .highlight_style(Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::LightBlue)
            );

            // Lay out timed events in the bottom left block.
            let table_timed = Table::new(
                s.timed
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
            ])
            .highlight_style(Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::LightBlue)
            );
            
            // Lay out untimed events in the bottom right block.
            let list_untimed = List::new(s.untimed
                .iter()
                .map(|event| ListItem::new(event.description.clone()))
                .collect::<Vec<ListItem>>()
            )
            .block(Block::default()
                .borders(Borders::ALL)
                .title("Untimed")
            )
            .highlight_style(Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::LightBlue)
            );

            term.render_stateful_widget(table_intervals, chunks[0], &mut state.intervals);
            term.render_stateful_widget(table_timed, chunks_bottom[0], &mut state.timed);
            term.render_stateful_widget(list_untimed, chunks_bottom[1], &mut state.untimed);
        })?;

        match state.focus {
            Focus::InputAdd => match event::read()? {
                event::Event::Key(keycode) => match keycode.code {
                    KeyCode::Esc => state.focus(Focus::None),
                    _ => {},
                },
                _ => {},
            },
            Focus::Intervals => match event::read()? {
                event::Event::Key(keycode) => match keycode.code {
                    KeyCode::Char('h') => state.focus(Focus::Timed),
                    KeyCode::Char('i') => state.focus(Focus::InputAdd),
                    KeyCode::Char('j') => s.scroll_down(),
                    KeyCode::Char('k') => s.scroll_up(),
                    KeyCode::Char('l') => state.focus(Focus::Untimed),
                    KeyCode::Char('q') => break,
                    KeyCode::Esc => state.focus(Focus::None),
                    _ => {},
                },
                _ => {},
            },
            Focus::Timed => match event::read()? {
                event::Event::Key(keycode) => match keycode.code {
                    KeyCode::Char('i') => state.focus(Focus::InputAdd),
                    KeyCode::Char('j') => s.scroll_down(),
                    KeyCode::Char('k') => s.scroll_up(),
                    KeyCode::Char('l') => state.focus(Focus::Untimed),
                    KeyCode::Char('q') => break,
                    KeyCode::Esc => state.focus(Focus::None),
                    _ => {},
                },
                _ => {},
            },
            Focus::Untimed => match event::read()? {
                event::Event::Key(keycode) => match keycode.code {
                    KeyCode::Char('h') => state.focus(Focus::Timed),
                    KeyCode::Char('i') => state.focus(Focus::InputAdd),
                    KeyCode::Char('j') => s.scroll_down(),
                    KeyCode::Char('k') => s.scroll_up(),
                    KeyCode::Char('q') => break,
                    KeyCode::Esc => state.focus(Focus::None),
                    _ => {},
                },
                _ => {},
            },
            Focus::None => match event::read()? {
                event::Event::Key(keycode) => match keycode.code {
                    KeyCode::Char('g') => state.focus(Focus::Intervals),
                    KeyCode::Char('h') => state.focus(Focus::Timed),
                    KeyCode::Char('i') => state.focus(Focus::InputAdd),
                    KeyCode::Char('l') => state.focus(Focus::Untimed),
                    _ => {},
                },
                _ => {},
            },
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
