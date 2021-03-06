# clamendar
task manager / calendar in Rust

## Purpose
This is mainly a learning project. I like Rust and want a simple way to manage all of my tasks, ongoing notes, and calendar events under a single roof. Thus, this. The current idea is to encode all of the tasks (tasks and calendar items) using ISO 8601 strings and a single description string. They will serialize to JSON and be manageable using an interactive terminal UI or a headless scripting interface.

### Why is it called clamendar?
Because Rust's mascot is a [crab][1], I was trying to find a good portmanteau of "calendar" and some oceanic creature. Whether I succeeded is up to you.

## Installation
From source. See `Cargo.toml` for dependencies.

## Usage

### Configuration
Compilation-time configuration in `src/config.rs`. There are no plans for runtime configuration.

### Invocation and Layout
Invoke `clamendar` in a terminal, provided you have installed it to your `PATH`. There are three panes. On top are intervals; use this for ongoing periods such as "spring break" or "second decade of existence". On the left are standard and repeating events. Here, track things like "history paper due" or "Mom's birthday". On the right are untimed events. Use this pane for reminders without due dates such as "read the next chapter of *The Rust Programming Language*".

### Controls
vi-inspired.

Key    | Effect                                                                            | Notes (below)
:------|:----------------------------------------------------------------------------------|:-------------
g      | Focuses the top pane.                                                             | 1
h      | Focuses the left pane.                                                            | 1
l      | Focuses the right pane.                                                           | 1
j      | Selects the item below the selected one, if one exists.                           | 1
k      | Selects the item above the selected one, if one exists.                           | 1
Up     | Moves the cursor to the beginning of the field.                                   | 2
Down   | Moves the cursor to the end of the field.                                         | 2
Left   | Moves the cursor left.                                                            | 2
Right  | Moves the cursor right.                                                           | 2
d      | Deletes the selected item. Issues a warning first.                                | 1
y      | Yanks (cuts) the selected item into the insertion buffer. Issues a warning first. | 1
i      | Focuses the insertion field and enters insert mode.                               | 1
Enter  | Attempts to add the event described in the insertion buffer.                      | 2
Escape | Focuses no pane, or exits insert mode, in which case the buffer is preserved.     |

1. Not available in insert mode
2. Only available in insert mode

### Insert Mode
Use the left/right arrow keys to move the cursor accordingly. 'Up' moves to the beginning of the field; 'Down' moves to the end. Note that currently, tab characters are always represented as single spaces. The insertion buffer is cleared only manually or when an event is successfully added.

#### Insert Format
I didn't want to have to select from a menu which type of event I'm inputting. As such, input relies on shallow knowledge of [ISO-8601 strings][2]. Valid format is as follows:

iso string | tab      | description (no tabs allowed)
:----------|:---------|:-----------------------------
optional   | required | optional, I guess

#### Insertion Examples
Substitute '(\t)' for a tab character.

String                                                | Pane             | Notes (below)
:-----------------------------------------------------|:-----------------|:-------------
`R/1970-01-01/1971-01-01(\t)Mom's birthday`           | Left (repeating) | 1
`R5/2021-01-01/2021-01-08(\t)Weekly meeting with Tod` | Left (repeating) | 2
`2011-01-01/2020-12-31(\t)second decade of existence` | Top              |
`2021-01-01T23:59(\t)history paper due`               | Left (standard)  |
`(\t)read the next chapter of TRPL`                   | Right            |


1. Eventually, `clamendar` should be able to advance this datetime string to, for instance, `R/2021-01-01/2022-01-01` for you. It cannot at the moment.
2. See (1). Once the five repetitions had occurred, the event would be deleted.

## TODO
- [ ] think about serializing to iCal instead of/in addition to JSON
- [ ] implement advancing logic for repeating events
- [x] commenting pass, especially for `src/config.rs`

## Hard Hat Required
This project is in progress. Feel free to bug me for information not found here.

[1]: https://www.rustacean.net/
[2]: https://en.wikipedia.org/wiki/ISO_8601
