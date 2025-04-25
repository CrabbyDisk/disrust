//GUI = gooey
use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::io;
use std::sync::mpsc::Receiver;

use std::time::Duration;
use std::time::Instant;

use crate::api::data::*;
use crate::ui::channels::App;
use crate::ui::{
    channels::DisplayMode::{ChannelMode, GuildMode},
    chat_box::{ChatBox, InputMode},
};

//Main loop
pub fn run<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    cbox: &mut ChatBox,
    gate_rx: &Receiver<GatewayResponse>,
) -> io::Result<()> {
    let tick_rate = Duration::from_millis(250);
    let mut last_tick = Instant::now();
    loop {
        match gate_rx.try_recv() {
            Ok(v) => app.react_to_gateway(&v),
            Err(_v) => {}
        }

        //Draws the screen. Comment out when debugging
        terminal.draw(|f| ui(f, app, cbox))?;
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        //Read input
        //CodeAesthetic would be upset
        //Have to use poll to avoid blocking
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match cbox.input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char('e') => cbox.toggle(),
                        KeyCode::Left => app.unselect(),
                        KeyCode::Down => app.next(),
                        KeyCode::Up => app.previous(),
                        KeyCode::Enter => app.enter_guild(),
                        KeyCode::Esc => app.leave_guild(),
                        _ => (),
                    },
                    InputMode::Editing => match key.code {
                        KeyCode::Enter => cbox.send_message(app),
                        KeyCode::Esc => cbox.toggle(),
                        KeyCode::Char(c) => cbox.input.push(c),
                        KeyCode::Backspace => {
                            cbox.input.pop();
                        }
                        _ => (),
                    },
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }
}

//Maybe make each block a function
//Sets up how the ui looks like
fn ui(f: &mut Frame, app: &mut App, cbox: &mut ChatBox) {
    //Wrapping block
    //Mandatory margin of 1+
    let wrapping_block = Block::default()
        .borders(Borders::ALL)
        .title("Disrust")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded);
    f.render_widget(wrapping_block, f.area());

    // this is all just defined boundaries used when drawing
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
        .split(f.area());

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(94), Constraint::Percentage(6)].as_ref())
        .split(chunks[1]);

    // Create the channels part
    let items = match app.mode {
        GuildMode => List::from(app.guilds.clone()),
        ChannelMode => List::from(app.channels.clone()),
    };

    let items = items
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Guilds and Channels"),
        )
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    // We can now render the item list
    // Displays channels or guilds depending on mode
    match app.mode {
        GuildMode => {
            f.render_stateful_widget(items, chunks[0], &mut app.guilds.state);
        }
        //Weird that let items isn't used, or so vscode thinks
        ChannelMode => {
            f.render_stateful_widget(items, chunks[0], &mut app.channels.state);
        }
    }

    // Could be better, a lot of cloning
    let title = app.get_current_title();
    let chat_messages = app.get_messages();

    //If there are messages, use those, if there aren't advertise
    if let Some(v) = chat_messages {
        let chat = List::from(v);
        let chat = chat.block(Block::default().borders(Borders::ALL).title(title));

        f.render_widget(chat, right_chunks[0]);
    } else {
        let ad = vec![ListItem::new(
            "Check my other projects on https://github.com/DvorakDwarf",
        )];
        let chat = List::new(ad).block(Block::default().borders(Borders::ALL).title(title));

        f.render_widget(chat, right_chunks[0]);
    }

    //The chat box is here
    let input = Paragraph::new(cbox.input.as_str())
        .style(match cbox.input_mode {
            InputMode::Normal => Style::default(),
            InputMode::Editing => Style::default().fg(Color::Yellow),
        })
        .block(Block::default().borders(Borders::ALL).title("Input"));
    f.render_widget(input, right_chunks[1]);

    match cbox.input_mode {
        InputMode::Normal => {} //hides cursor
        InputMode::Editing => {
            //Set cursor as visible and move to right spot
            f.set_cursor_position((
                // Put cursor past the end of the input text
                right_chunks[1].x + cbox.input.len() as u16 + 1,
                // Move one line down, from the border to the input line
                right_chunks[1].y + 1,
            ))
        }
    }
}
