/// Contains constants that the user might want to modify.

/// The prompt that appears at the beginning of the insert box when in insert mode.
pub const ADD_PROMPT: &str = "Add: >";
/// Keep ADD_PROMPT_LEN up to date to ensure proper cursor positioning in insert mode.
pub const ADD_PROMPT_LEN: u16 = 6;
/// Column width for the "Time" columns of relevant panes. You may want to adjust this according
/// to the format used for printing times in the interface.
pub const COL_TIME_WIDTH: u16 = 12;
/// File into which to backup events during every session. Point the constant to an empty &str to
/// disable backups.
pub const FILEPATH_BACKUP: &str = "/home/ty/code/clamendar/events.json.bak";
/// File to use for de/serialization. Must be an absolute path, I think.
pub const FILEPATH: &str = "/home/ty/code/clamendar/events.json";
/// Preferred output Year, Month, Day format for printing only the date.
pub const YMD: &str = "%m-%d";
/// Preferred output Year, Month, Day, Hour, Minute format for printing the date and time.
pub const YMDHM: &str = "%m-%d %R";
