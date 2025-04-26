mod api;
mod ui;
use crate::api::data::*;
use api::gateway_thread;
use tokio::sync::mpsc;
//has all the structs used
use ui::{channels::App, chat_box::ChatBox, gui::run};

#[tokio::main]
async fn main() {
    println!("Please paste in your token. If you don't know what that is, please google");

    let mut token = String::new();
    std::io::stdin()
        .read_line(&mut token)
        .expect("Could not read input");
    token.pop(); //get rid of \n on the end

    let conn = Connection::new(&token);

    let mut terminal = ratatui::init();

    let (tx, mut rx) = mpsc::unbounded_channel();

    // conn.auth.1 is the token
    gateway_thread::start_thread(tx, &conn.auth.1).await;

    let guilds = rx.recv().await.unwrap().guilds;

    let mut app = App::new(guilds, conn);
    let mut cbox = ChatBox::new();

    let result = run(&mut terminal, &mut app, &mut cbox, rx).await;

    ratatui::restore();
    if let Err(err) = result {
        println!("{:?}", err)
    }
}
