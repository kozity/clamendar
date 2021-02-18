mod config;
mod error;
mod state;

use chrono::{DateTime, Local, LocalResult, Timelike, TimeZone};
use clamendar::{self, Event, Interval};
use crate::{
    config::*,
    error::Error,
    state::{ Focus, State },
};
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
    widgets::{Block, Borders, List, ListItem, Paragraph, Row, Table},
};

fn main() -> Result<(), Error> {
    let mut s = State::default();
    // split all deserialized events into vectors: one per pane
    for event in deserialize()? {
        match (&event.start, &event.interval) {
            (Some(_), Interval::None)
                | (Some(_), Interval::RepDefinite { .. })
                | (Some(_), Interval::RepIndefinite(_))
            => s.timed.push(event),
            (Some(_), _) => s.intervals.push(event),
            (None, _) => s.untimed.push(event),
        }
    }
    // events are sorted first chronologically, then alphabetically by description
    s.intervals.sort_unstable();
    s.timed.sort_unstable();
    s.untimed.sort_unstable();

    if !FILEPATH_BACKUP.is_empty() { fs::copy(FILEPATH, FILEPATH_BACKUP)?; }

    let mut terminal = Terminal::new(
        CrosstermBackend::new(
            io::stdout()
        )
    )?;
    terminal::enable_raw_mode()?;
    terminal.clear()?;

    // the main loop renders, then handles input
    loop {
        terminal.draw(|term| {
            let main_rectangle = term.size();
            // split the screen vertically
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(6),
                        Constraint::Min(0),
                        Constraint::Length(3),
                    ]
                    .as_ref()
                )
                .split(main_rectangle);
            // split the middle vertical section into left and right panes
            let chunks_bottom = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [
                        Constraint::Percentage(60),
                        Constraint::Percentage(40),
                    ]
                )
                .split(chunks[1]);

            // lay out intervals in the top block
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
                Constraint::Length(COL_TIME_WIDTH),
                Constraint::Min(0),
            ])
            .highlight_style(Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::LightBlue)
            );

            // lay out timed events in the bottom left block
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
                Constraint::Length(COL_TIME_WIDTH),
                Constraint::Min(0),
            ])
            .highlight_style(Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::LightBlue)
            );
            
            // lay out untimed events in the bottom right block
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

            // lay out insert box
            let text = match s.focus {
                Focus::InputAdd => Paragraph::new(format!("{}{}", ADD_PROMPT, s.buffer.replace('\t', " "))),
                _ => match s.last_error {
                    Some(Error::DeletionWarning) => Paragraph::new("Warning: press 'd' again to delete (irreversible)."),
                    Some(Error::InvalidIso) => Paragraph::new("Error: the string was not properly formatted."),
                    Some(Error::InvalidRecord) => Paragraph::new("Error: input: \"[iso string]\\t[description]\""),
                    Some(Error::InvalidTime) => Paragraph::new("Error: the time entered was invalid or not specific enough."),
                    Some(Error::NoInfo) => Paragraph::new("The event contained no information, so was not added."),
                    Some(Error::YankWarning) => Paragraph::new("Warning: press 'y' again to yank selected event into buffer."),
                    _ => Paragraph::new(""),
                },
            }.block(Block::default().borders(Borders::ALL));

            term.render_stateful_widget(table_intervals, chunks[0], &mut s.intervals_state);
            term.render_stateful_widget(table_timed, chunks_bottom[0], &mut s.timed_state);
            term.render_stateful_widget(list_untimed, chunks_bottom[1], &mut s.untimed_state);
            term.render_widget(text, chunks[2]);
        })?;

        // this is the event-handling half of the main loop; keybindings are matched individually
        // in the match arm for each "Focus" (insert, or whichever pane is selected, if any)
        match s.focus {
            Focus::InputAdd => {
                let len: u16 = if s.cursor_offset > u16::MAX.into() {
                    u16::MAX
                } else {
                    s.cursor_offset as u16
                };
                terminal.set_cursor(ADD_PROMPT_LEN + len + 1, terminal.size()?.height - 2)?;
                terminal.show_cursor()?;

                match event::read()? {
                    event::Event::Key(keycode) => match keycode.code {
                        KeyCode::Backspace => if s.cursor_offset > 0 {
                            s.cursor_offset -= 1;
                            s.buffer.remove(s.cursor_offset);
                        },
                        KeyCode::Char(c) => {
                            s.buffer.insert(s.cursor_offset, c);
                            s.cursor_offset += 1;
                        },
                        KeyCode::Delete => if s.cursor_offset < s.buffer.len() {
                            s.buffer.remove(s.cursor_offset);
                        },
                        KeyCode::Down => s.cursor_end(),
                        KeyCode::Enter => {
                            match s.add_event_from_buffer() {
                                err @ Err(Error::InvalidIso)
                                    | err @ Err(Error::InvalidRecord)
                                    | err @ Err(Error::InvalidTime)
                                    | err @ Err(Error::NoInfo)
                                    => s.last_error = Some(err.unwrap_err()),
                                _ => s.last_error = None,
                            }
                            s.focus(Focus::None);
                        },
                        KeyCode::Esc => s.focus(Focus::None),
                        KeyCode::Left => s.cursor_left(),
                        KeyCode::Right => s.cursor_right(),
                        KeyCode::Tab => {
                            s.buffer.insert(s.cursor_offset, '\t');
                            s.cursor_offset += 1;
                        },
                        KeyCode::Up => s.cursor_beginning(),
                        _ => {},
                    },
                    _ => {},
                }
            },
            Focus::Intervals => match event::read()? {
                event::Event::Key(keycode) => match keycode.code {
                    KeyCode::Char('d') => s.delete_selected(),
                    KeyCode::Char('h') => s.focus(Focus::Timed),
                    KeyCode::Char('i') => {
                        s.focus(Focus::InputAdd);
                        terminal.show_cursor()?;
                        terminal.set_cursor(ADD_PROMPT_LEN, 0)?;
                    },
                    KeyCode::Char('j') => s.scroll_down(),
                    KeyCode::Char('k') => s.scroll_up(),
                    KeyCode::Char('l') => s.focus(Focus::Untimed),
                    KeyCode::Char('q') => break,
                    KeyCode::Char('y') => s.yank_selected(),
                    KeyCode::Esc => match s.last_error {
                        Some(Error::DeletionWarning) => s.last_error = None,
                        _ => s.focus(Focus::None),
                    },
                    _ => {},
                },
                _ => {},
            },
            Focus::Timed => match event::read()? {
                event::Event::Key(keycode) => match keycode.code {
                    KeyCode::Char('d') => s.delete_selected(),
                    KeyCode::Char('g') => s.focus(Focus::Intervals),
                    KeyCode::Char('i') => {
                        s.focus(Focus::InputAdd);
                        terminal.set_cursor(ADD_PROMPT_LEN, 0)?;
                    },
                    KeyCode::Char('j') => s.scroll_down(),
                    KeyCode::Char('k') => s.scroll_up(),
                    KeyCode::Char('l') => s.focus(Focus::Untimed),
                    KeyCode::Char('q') => break,
                    KeyCode::Char('y') => s.yank_selected(),
                    KeyCode::Esc => match s.last_error {
                        Some(Error::DeletionWarning) => s.last_error = None,
                        _ => s.focus(Focus::None),
                    },
                    _ => {},
                },
                _ => {},
            },
            Focus::Untimed => match event::read()? {
                event::Event::Key(keycode) => match keycode.code {
                    KeyCode::Char('d') => s.delete_selected(),
                    KeyCode::Char('g') => s.focus(Focus::Intervals),
                    KeyCode::Char('h') => s.focus(Focus::Timed),
                    KeyCode::Char('i') => {
                        s.focus(Focus::InputAdd);
                        terminal.set_cursor(ADD_PROMPT_LEN, 0)?;
                    },
                    KeyCode::Char('j') => s.scroll_down(),
                    KeyCode::Char('k') => s.scroll_up(),
                    KeyCode::Char('q') => break,
                    KeyCode::Char('y') => s.yank_selected(),
                    KeyCode::Esc => match s.last_error {
                        Some(Error::DeletionWarning) => s.last_error = None,
                        _ => s.focus(Focus::None),
                    },
                    _ => {},
                },
                _ => {},
            },
            Focus::None => match event::read()? {
                event::Event::Key(keycode) => match keycode.code {
                    KeyCode::Char('g') => s.focus(Focus::Intervals),
                    KeyCode::Char('h') => s.focus(Focus::Timed),
                    KeyCode::Char('i') => {
                        s.focus(Focus::InputAdd);
                        terminal.set_cursor(ADD_PROMPT_LEN, 0)?;
                    },
                    KeyCode::Char('l') => s.focus(Focus::Untimed),
                    KeyCode::Char('q') => break,
                    KeyCode::Esc => s.last_error = None,
                    _ => {},
                },
                _ => {},
            },
        }
    }

    terminal::disable_raw_mode()?;
    terminal.clear()?;
    terminal.set_cursor(0, 0)?;
    serialize(s.take_all())
}

/// Attempts to parse a chrono DateTime from the given string using the Local timezone.
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

/// Formats a string from an existing event so that events yanked into buffer have enough
/// information to be valid.
fn event_to_record(event: Event) -> String {
    const ISO_FULL: &str = "%FT%R";
    match (event.start, event.interval) {
        (None, _) => format!(
            "\t{}",
            event.description
        ),
        (_, Interval::RepDefinite { occurrences, end }) => format!(
            "R{}/{}/{}\t{}",
            occurrences,
            event.start.unwrap().format(ISO_FULL),
            end.format(ISO_FULL),
            event.description
        ),
        (_, Interval::RepIndefinite(end)) => format!(
            "R/{}/{}\t{}",
            event.start.unwrap().format(ISO_FULL),
            end.format(ISO_FULL),
            event.description
        ),
        (_, Interval::Standard(end)) => format!(
            "{}/{}\t{}",
            event.start.unwrap().format(ISO_FULL),
            end.format(ISO_FULL),
            event.description
        ),
        (_, Interval::None) => format!(
            "{}\t{}",
            event.start.unwrap().format(ISO_FULL),
            event.description
        ),
    }
}

/// Attempts to deserialize a vector of Events from FILEPATH.
fn deserialize() -> Result<Vec<Event>, Error> {
    let file = fs::read_to_string(FILEPATH)?;
    Ok(serde_json::from_str(&file)?)
}

/// Returns a string representing the end-time of an interval. Returns None for events without an
/// end-time.
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

/// Attempts to serialize a vector of Events into FILEPATH.
fn serialize(events: Vec<Event>) -> Result<(), Error> {
    Ok(fs::write(FILEPATH, &serde_json::to_vec(&events)?)?)
}

/// Returns a string representing the start-time of an interval. Returns None for untimed events.
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
