// TODO: replace start_time()'s with functionality native to main.rs
// TODO: add ability to delete events

use chrono::{DateTime, Local, LocalResult, Timelike, TimeZone};
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
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Row, Table, TableState},
};

const ADD_PROMPT: &str = "Add: >";
const ADD_PROMPT_LEN: u16 = 6;
const FILEPATH: &str = "./events.json";
const YMD: &str = "%F";
const YMDHM: &str = "%FT%R";

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
    last_error: Option<Error>,
    timed: Vec<Event>,
    timed_offset: usize,
    timed_state: TableState,
    untimed: Vec<Event>,
    untimed_offset: usize,
    untimed_state: ListState,
}

impl State {
    fn add_event_from_buffer(&mut self) -> Result<(), Error> {
        if self.buffer.is_empty() { return Err(Error::NoInfo); }
        if !self.buffer.contains('\t') { return Err(Error::InvalidRecord); }
        let mut event = Event {
            start: None,
            interval: Interval::None,
            description: String::new(),
        };
        let mut halves = self.buffer.split('\t');
        match halves.next() {
            Some("") => {}, // leave event.start empty.
            Some(iso) => {
                let mut tokens = iso.split('/');
                match (tokens.next(), tokens.next(), tokens.next()) {
                    (Some(repetition), Some(start), Some(end)) => {
                        match repetition.strip_prefix('R') {
                            Some("") => event.interval = Interval::RepIndefinite(datetime_from_iso(end)?),
                            Some(string) => {
                                let occurrences = match string.parse::<usize>() {
                                    Ok(num) => num,
                                    Err(_) => return Err(Error::InvalidIso),
                                };
                                event.interval = Interval::RepDefinite {
                                    occurrences,
                                    end: datetime_from_iso(end)?,
                                };
                            },
                            None => return Err(Error::InvalidIso),
                        }
                        event.start = Some(datetime_from_iso(start)?);
                    },
                    (Some(start), Some(end), None) => {
                        event.start = Some(datetime_from_iso(start)?);
                        event.interval = Interval::Standard(datetime_from_iso(end)?);
                    },
                    (Some(start), None, None) => {
                        event.start = Some(datetime_from_iso(start)?);
                        event.interval = Interval::None;
                    },
                    _ => {},
                }
            },
            None => {}, // leave event.start empty
        }
        match halves.next() {
            Some(string) => event.description.push_str(string.trim()),
            None => {}, // leave event.description empty
        }
        if event.start == None && event.description == "" { return Err(Error::NoInfo); }
        match (event.start, &event.interval) {
            (None, _) => self.untimed.push(event),
            (Some(_), Interval::RepDefinite { .. })
            | (Some(_), Interval::RepIndefinite(_))
            | (Some(_), Interval::Standard(_)) => {
                self.intervals.push(event);
                self.intervals.sort_unstable();
            },
            (Some(_), Interval::None) => {
                self.timed.push(event);
                self.timed.sort_unstable();
            },
        }
        self.buffer.clear();
        Ok(())
    }

    fn focus(&mut self, target: Focus) -> Result<(), Error> {
        match self.focus {
            Focus::Intervals => self.intervals_state.select(None),
            Focus::Timed => self.timed_state.select(None),
            Focus::Untimed => self.untimed_state.select(None),
            _ => {},
        }
        match target {
            Focus::Intervals => if !self.intervals.is_empty() { self.intervals_state.select(Some(self.intervals_offset)) },
            Focus::Timed => if !self.timed.is_empty() { self.timed_state.select(Some(self.timed_offset)) },
            Focus::Untimed => if !self.untimed.is_empty() { self.untimed_state.select(Some(self.untimed_offset)) },
            _ => {},
        }
        self.focus = target;
        Ok(())
    }

    fn scroll_down(&mut self) {
        match self.focus {
            Focus::Intervals => match self.intervals_state.selected() {
                Some(selected) if selected < self.intervals.len() - 1 => {
                    self.intervals_offset = selected + 1;
                    self.intervals_state.select(Some(self.intervals_offset));
                },
                _ => {},
            },
            Focus::Timed => match self.timed_state.selected() {
                Some(selected) if selected < self.timed.len() - 1 => {
                    self.timed_offset = selected + 1;
                    self.timed_state.select(Some(self.timed_offset));
                },
                _ => {},
            },
            Focus::Untimed => match self.untimed_state.selected() {
                Some(selected) if selected < self.untimed.len() - 1 => {
                    self.untimed_offset = selected + 1;
                    self.untimed_state.select(Some(self.untimed_offset));
                },
                _ => {},
            },
            _ => {},
        }
    }

    fn scroll_up(&mut self) {
        match self.focus {
            Focus::Intervals => match self.intervals_state.selected() {
                Some(selected) if selected > 0 => {
                    self.intervals_offset = selected - 1;
                    self.intervals_state.select(Some(self.intervals_offset));
                },
                _ => {},
            },
            Focus::Timed => match self.timed_state.selected() {
                Some(selected) if selected > 0 => {
                    self.timed_offset = selected - 1;
                    self.timed_state.select(Some(self.timed_offset));
                },
                _ => {},
            },
            Focus::Untimed => match self.untimed_state.selected() {
                Some(selected) if selected > 0 => {
                    self.untimed_offset = selected - 1;
                    self.untimed_state.select(Some(self.untimed_offset));
                },
                _ => {},
            },
            _ => {},
        }
    }

    fn take_all(&mut self) -> Vec<Event> {
        let mut vec: Vec<Event> = Vec::new();
        vec.append(&mut self.intervals);
        vec.append(&mut self.timed);
        vec.append(&mut self.untimed);
        vec
    }
}

#[derive(Debug)]
enum Error {
    Crossterm(crossterm::ErrorKind),
    InvalidIso,
    InvalidRecord,
    InvalidTime,
    Io(io::Error),
    NoInfo,
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
        last_error: None,
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

    let mut terminal = Terminal::new(
        CrosstermBackend::new(
            io::stdout()
        )
    )?;
    terminal::enable_raw_mode()?;
    terminal.clear()?;

    loop {
        terminal.draw(|term| {
            let main_rectangle = term.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(10),
                        Constraint::Min(0),
                        Constraint::Length(3),
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
                            end_time(&event).unwrap(),
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
                            start_time(&event).unwrap(),
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

            // Lay out bottom text line.
            let text = match s.focus {
                Focus::InputAdd => Paragraph::new(format!("{}{}", ADD_PROMPT, s.buffer.replace('\t', " "))),
                _ => match s.last_error {
                    Some(Error::InvalidIso) => Paragraph::new("Error: the string was not properly formatted."),
                    Some(Error::InvalidRecord) => Paragraph::new("Error: input: \"[iso string]\\t[description]\""),
                    Some(Error::InvalidTime) => Paragraph::new("Error: the time entered was invalid or not specific enough."),
                    Some(Error::NoInfo) => Paragraph::new("The event contained no information, so was not added."),
                    _ => Paragraph::new(""),
                },
            }.block(Block::default().borders(Borders::ALL));

            term.render_stateful_widget(table_intervals, chunks[0], &mut s.intervals_state);
            term.render_stateful_widget(table_timed, chunks_bottom[0], &mut s.timed_state);
            term.render_stateful_widget(list_untimed, chunks_bottom[1], &mut s.untimed_state);
            term.render_widget(text, chunks[2]);
        })?;

        match s.focus {
            Focus::InputAdd => {
                let len: u16 = if s.buffer.len() > u16::MAX.into() {
                    u16::MAX
                } else {
                    s.buffer.len() as u16
                };
                terminal.set_cursor(ADD_PROMPT_LEN + len + 1, terminal.size()?.height - 2)?;
                terminal.show_cursor()?;

                match event::read()? {
                    event::Event::Key(keycode) => match keycode.code {
                        KeyCode::Backspace => { s.buffer.pop(); },
                        KeyCode::Char(c) => s.buffer.push(c),
                        KeyCode::Enter => {
                            match s.add_event_from_buffer() {
                                err @ Err(Error::InvalidIso)
                                    | err @ Err(Error::InvalidRecord)
                                    | err @ Err(Error::InvalidTime)
                                    | err @ Err(Error::NoInfo)
                                    => s.last_error = Some(err.unwrap_err()),
                                _ => s.last_error = None,
                            }
                            s.focus(Focus::None)?;
                        },
                        KeyCode::Esc => s.focus(Focus::None)?,
                        KeyCode::Tab => s.buffer.push('\t'),
                        _ => {},
                    },
                    _ => {},
                }
            },
            Focus::Intervals => match event::read()? {
                event::Event::Key(keycode) => match keycode.code {
                    KeyCode::Char('h') => s.focus(Focus::Timed)?,
                    KeyCode::Char('i') => {
                        s.focus(Focus::InputAdd)?;
                        terminal.show_cursor()?;
                        terminal.set_cursor(ADD_PROMPT_LEN, 0)?;
                    },
                    KeyCode::Char('j') => s.scroll_down(),
                    KeyCode::Char('k') => s.scroll_up(),
                    KeyCode::Char('l') => s.focus(Focus::Untimed)?,
                    KeyCode::Char('q') => break,
                    KeyCode::Esc => s.focus(Focus::None)?,
                    _ => {},
                },
                _ => {},
            },
            Focus::Timed => match event::read()? {
                event::Event::Key(keycode) => match keycode.code {
                    KeyCode::Char('g') => s.focus(Focus::Intervals)?,
                    KeyCode::Char('i') => {
                        s.focus(Focus::InputAdd)?;
                        terminal.set_cursor(ADD_PROMPT_LEN, 0)?;
                    },
                    KeyCode::Char('j') => s.scroll_down(),
                    KeyCode::Char('k') => s.scroll_up(),
                    KeyCode::Char('l') => s.focus(Focus::Untimed)?,
                    KeyCode::Char('q') => break,
                    KeyCode::Esc => s.focus(Focus::None)?,
                    _ => {},
                },
                _ => {},
            },
            Focus::Untimed => match event::read()? {
                event::Event::Key(keycode) => match keycode.code {
                    KeyCode::Char('g') => s.focus(Focus::Intervals)?,
                    KeyCode::Char('h') => s.focus(Focus::Timed)?,
                    KeyCode::Char('i') => {
                        s.focus(Focus::InputAdd)?;
                        terminal.set_cursor(ADD_PROMPT_LEN, 0)?;
                    },
                    KeyCode::Char('j') => s.scroll_down(),
                    KeyCode::Char('k') => s.scroll_up(),
                    KeyCode::Char('q') => break,
                    KeyCode::Esc => s.focus(Focus::None)?,
                    _ => {},
                },
                _ => {},
            },
            Focus::None => match event::read()? {
                event::Event::Key(keycode) => match keycode.code {
                    KeyCode::Char('g') => s.focus(Focus::Intervals)?,
                    KeyCode::Char('h') => s.focus(Focus::Timed)?,
                    KeyCode::Char('i') => {
                        s.focus(Focus::InputAdd)?;
                        terminal.set_cursor(ADD_PROMPT_LEN, 0)?;
                    },
                    KeyCode::Char('l') => s.focus(Focus::Untimed)?,
                    KeyCode::Char('q') => break,
                    KeyCode::Esc => s.last_error = None,
                    _ => {},
                },
                _ => {},
            },
        }
    }

    terminal::disable_raw_mode()?;
    serialize(s.take_all())
}

fn datetime_from_iso(string: &str) -> Result<DateTime<Local>, Error> {
    let mut tokens = string.split(&['-', 'T', ':'][..]);
    let date = match (tokens.next(), tokens.next(), tokens.next()) {
        (Some(year), Some(month), Some(day)) => {
            let year = match year.parse::<i32>() {
                Ok(num) => num,
                Err(_) => return Err(Error::InvalidIso),
            };
            let month = match month.parse::<u32>() {
                Ok(num) => num,
                Err(_) => return Err(Error::InvalidIso),
            };
            let day = match day.parse::<u32>() {
                Ok(num) => num,
                Err(_) => return Err(Error::InvalidIso),
            };
            Local.ymd_opt(year, month, day)
        },
        _ => return Err(Error::InvalidIso),
    };
    let datetime = match date {
        LocalResult::Single(date) => date.and_hms_opt(
            match tokens.next() {
                Some(hour) => match hour.parse::<u32>() {
                    Ok(num) => num,
                    Err(_) => return Err(Error::InvalidIso),
                },
                _ => 0,
            },
            match tokens.next() {
                Some(minute) => match minute.parse::<u32>() {
                    Ok(num) => num,
                    Err(_) => return Err(Error::InvalidIso),
                },
                _ => 0,
            },
            match tokens.next() {
                Some(second) => match second.parse::<u32>() {
                    Ok(num) => num,
                    Err(_) => return Err(Error::InvalidIso),
                },
                _ => 0,
            }
        ),
        _ => return Err(Error::InvalidTime),
    };
    match datetime {
        Some(d) => Ok(d),
        None => Err(Error::InvalidTime),
    }
}

fn deserialize() -> Result<Vec<Event>, Error> {
    let file = fs::read_to_string(FILEPATH)?;
    Ok(serde_json::from_str(&file)?)
}

fn end_time(event: &Event) -> Option<String> {
    match event.interval {
        Interval::RepDefinite { end, .. }
            | Interval::RepIndefinite(end)
            | Interval::Standard(end)
        => {
            let time = end.time();
            if time.hour() == 0 && time.minute() == 0 {
                Some(end.format(YMD).to_string())
            } else {
                Some(end.format(YMDHM).to_string())
            }
        },
        Interval::None => None,
    }
}

fn serialize(events: Vec<Event>) -> Result<(), Error> {
    Ok(fs::write(FILEPATH, &serde_json::to_vec(&events)?)?)
}

fn start_time(event: &Event) -> Option<String> {
    match event.start {
        Some(datetime) => {
            let time = datetime.time();
            if time.hour() == 0 && time.minute() == 0 {
                Some(datetime.format(YMD).to_string())
            } else {
                Some(datetime.format(YMDHM).to_string())
            }
        },
        None => None,
    }
}
