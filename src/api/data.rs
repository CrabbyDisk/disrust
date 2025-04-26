//defining structs here for convenience and to clear up api.rs

use reqwest::Client;
use serde_json::Value;

fn get_length(list: &serde_json::Value) -> usize {
    let the_length = list.as_array();
    match the_length {
        Some(_v) => (),
        None => {
            panic!("TRIED TO GET LENGTH OF AN EMPTY RESPONSE")
        }
    }
    the_length.unwrap().len()
}

#[derive(Debug, Clone)]
pub struct Connection {
    pub auth: (String, String),
    pub client: Client,
}

impl Connection {
    pub fn new(token: &str) -> Connection {
        //This is a header with the token for authorization of api calls
        //Header required to pass as a user
        let auth = ("authorization".to_string(), token.to_string());
        let client = Client::new();

        Connection { auth, client }
    }
}

#[derive(Clone, Debug)]
pub struct GatewayResponse {
    pub operation: String,
    pub message: Msg,
    pub guilds: Vec<Guild>,
}

impl GatewayResponse {
    pub fn msg_create(message: Msg) -> GatewayResponse {
        GatewayResponse {
            operation: "MESSAGE_CREATE".to_string(),
            message,
            guilds: Vec::new(),
        }
    }

    //Send initial data like guilds
    pub fn ready(guilds: Vec<Guild>) -> GatewayResponse {
        GatewayResponse {
            operation: "READY".to_string(),
            message: Msg::new(),
            guilds,
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct User {
    pub id: u64,
    pub name: String,
    pub discriminator: String,
}

impl User {
    pub fn new() -> User {
        User {
            id: 111,
            name: "Dev".to_string(),
            discriminator: "0001".to_string(),
        }
    }
    pub fn from(author: &Value) -> User {
        let id = author["id"].as_str().unwrap().parse().unwrap();
        let name = author["username"].as_str().unwrap().to_string();
        let discriminator = author["discriminator"].as_str().unwrap().to_string();

        User {
            id,
            name,
            discriminator,
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Guild {
    pub id: u64,
    pub name: String,
    pub channels: Vec<Channel>,
}

impl Guild {
    pub fn from_list(event: &Value) -> Vec<Self> {
        //VERY UGLY + WRAPPER DUPE. Fix eventually
        let guild_vc = String::from("2");
        let category = String::from("4");
        let announcement_thread = String::from("10");
        let public_thread = String::from("11");
        let private_thread = String::from("12");
        let guild_stage_vc = String::from("13");
        let guild_directory = String::from("14");
        let guild_forum = String::from("15");

        let ignored_channels = Vec::from([
            guild_vc,
            category,
            announcement_thread,
            public_thread,
            private_thread,
            guild_stage_vc,
            guild_directory,
            guild_forum,
        ]);

        let guilds = event["guilds"].as_array().unwrap();
        guilds
            .iter()
            .map(|guild| Guild {
                id: guild["id"].as_str().unwrap().parse().unwrap(),
                name: guild["name"].as_str().unwrap().to_string(),
                channels: guild["channels"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(Channel::from)
                    .filter(|channel| !ignored_channels.contains(&channel.channel_type))
                    .collect(),
            })
            .collect()
    }

    //get rid of eventually
    pub fn from_partial(event: &Value) -> Guild {
        let id = event["id"].as_str().unwrap().parse().unwrap();
        let name = event["name"].as_str().unwrap().to_string();

        Guild {
            id,
            name,
            channels: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Channel {
    pub id: u64,
    pub name: String,
    pub channel_type: String,
}

impl Channel {
    pub fn from(event: &Value) -> Channel {
        let id = event["id"].as_str().unwrap().parse().unwrap();
        let name = event["name"].as_str().unwrap().to_string();
        let channel_type = event["type"].as_i64().unwrap().to_string();

        Channel {
            id,
            name,
            channel_type,
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Msg {
    pub id: u64,
    pub channel_id: u64,
    pub user: User,
    pub content: String,
}

impl Msg {
    pub fn new() -> Msg {
        Msg {
            id: 222,
            channel_id: 333,
            user: User::new(),
            content: "Unable to open a channel without proper permission".to_string(),
        }
    }
    //Might not work for every event in mind ??
    pub fn from(event: &Value) -> Msg {
        let id = event["id"].as_str().unwrap().parse().unwrap();
        let channel_id = event["channel_id"].as_str().unwrap().parse().unwrap();

        let author = &event["author"];
        let user = User::from(author);

        let content = event["content"].as_str().unwrap().to_string();

        Msg {
            id,
            channel_id,
            user,
            content,
        }
    }
}
