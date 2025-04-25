mod api;
mod ui;
use crate::api::data::*;
use api::gateway_thread;
//has all the structs used
use ui::{channels::App, chat_box::ChatBox, gui::run};

fn main() {
    println!("Please paste in your token. If you don't know what that is, please google");

    let mut token = String::new();
    std::io::stdin()
        .read_line(&mut token)
        .expect("Could not read input");
    token.pop(); //get rid of \n on the end

    let conn = Connection::new(&token);

    let mut terminal = ratatui::init();

    // conn.auth.1 is the token
    let gate_rx = gateway_thread::start_thread(&conn.auth.1);
    let guilds = gate_rx.recv().unwrap().guilds;

    let mut app = App::new(guilds, conn);
    let mut cbox = ChatBox::new();
    let result = run(&mut terminal, &mut app, &mut cbox, &gate_rx);

    ratatui::restore();
    if let Err(err) = result {
        println!("{:?}", err)
    }
}
