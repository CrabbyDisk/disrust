use std::collections::HashMap;

use crate::api::{data::*, wrapper};

use super::stateful_list::StatefulList;

#[derive(Debug)]
pub enum DisplayMode {
    GuildMode,
    ChannelMode,
}

pub struct App {
    pub channels: StatefulList<Channel>,
    pub guilds: StatefulList<Guild>,
    pub loaded_channels: HashMap<Channel, StatefulList<Msg>>,
    pub mode: DisplayMode,
    pub conn: Connection,
}

impl App {
    //build new app
    pub fn new(guilds: Vec<Guild>, conn: Connection) -> App {
        App {
            channels: StatefulList::from(Vec::new()),
            guilds: StatefulList::from(guilds),
            loaded_channels: HashMap::new(),
            mode: DisplayMode::GuildMode,
            conn,
        }
    }

    //Very awkward
    //Use enums and strum crate
    pub fn react_to_gateway(&mut self, gate_response: &GatewayResponse) {
        match gate_response.operation.as_str() {
            "MESSAGE_CREATE" => {
                let mut channel_found = Vec::new();
                let gate_channel_id = &gate_response.message.channel_id;
                for key in self.loaded_channels.keys() {
                    // println!("current channel id: {}, looking for: {}", &key.id, gate_channel_id);
                    let channel_id = &key.id;
                    if channel_id == gate_channel_id {
                        channel_found.push(key.clone());
                    }
                }

                for key in channel_found {
                    let mut old_messages = self.loaded_channels[&key].clone();
                    old_messages.items.push(gate_response.message.clone());
                    //updates the messages with the new one
                    self.loaded_channels.insert(key, old_messages);
                    // dbg!(&self.loaded_channels);
                }
            }
            "READY" => {
                dbg!(&gate_response.guilds);
            }
            _ => (),
        }
    }

    pub fn enter_guild(&mut self) {
        let current_guild = self.get_guild();
        let channels = current_guild.channels;

        self.channels = StatefulList::from(channels);
        self.mode = DisplayMode::ChannelMode;
    }

    pub fn leave_guild(&mut self) {
        self.mode = DisplayMode::GuildMode;
    }

    //get current selected channel object
    pub fn get_channel(&mut self) -> Channel {
        let index = self.channels.state.selected();
        let index = index.unwrap_or_default();
        self.channels.items[index].clone()
    }

    pub fn get_guild(&mut self) -> Guild {
        let index = self.guilds.state.selected();
        let index = index.unwrap_or_default();

        self.guilds.items[index].clone()
    }

    pub fn get_current_title(&mut self) -> String {
        match self.mode {
            DisplayMode::GuildMode => self.get_guild().name,
            DisplayMode::ChannelMode => self.get_channel().name,
        }
    }

    //CLONES EVERYTIME, PROBABLY SLOW
    pub fn get_messages(&mut self) -> Option<StatefulList<Msg>> {
        match self.mode {
            DisplayMode::GuildMode => return None,
            DisplayMode::ChannelMode => {}
        }

        let current_channel = self.get_channel();

        match self.loaded_channels.contains_key(&current_channel) {
            true => Some(self.loaded_channels[&current_channel].clone()),
            false => None,
        }
    }

    //Moves cursor down
    pub async fn next(&mut self) {
        match self.mode {
            DisplayMode::GuildMode => {
                self.guilds.next();
                return;
            }
            DisplayMode::ChannelMode => {
                self.channels.next();
            }
        }

        let current_channel = self.get_channel();
        //Check whether the channel has already been loaded
        //Don't wanna spam discord
        if !(self.loaded_channels.contains_key(&current_channel)) {
            let messages = wrapper::messages(&self.conn, &current_channel).await;

            match messages {
                Ok(v) => {
                    self.loaded_channels
                        .insert(current_channel, StatefulList::from(v));
                }
                Err(_) => {
                    self.loaded_channels
                        .insert(current_channel, StatefulList::from(vec![Msg::new()]));
                }
            }
        }
    }

    //Moves cursor up
    pub async fn previous(&mut self) {
        match self.mode {
            DisplayMode::GuildMode => {
                self.guilds.previous();
                return;
            }
            DisplayMode::ChannelMode => {
                self.channels.previous();
            }
        }

        let current_channel = self.get_channel();
        //Check whether the channel has already been loaded
        //Don't wanna spam discord
        if !(self.loaded_channels.contains_key(&current_channel)) {
            let messages = wrapper::messages(&self.conn, &current_channel).await;

            match messages {
                Ok(v) => {
                    self.loaded_channels
                        .insert(current_channel, StatefulList::from(v));
                }
                Err(_) => {
                    self.loaded_channels
                        .insert(current_channel, StatefulList::from(vec![Msg::new()]));
                }
            }
        }
    }

    pub fn unselect(&mut self) {
        self.channels.state.select(None);
    }
}
