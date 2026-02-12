pub enum Message {
    Close,
    Text {
        message: String,
        destinations: TextDestination,
    },
    Auth {
        id: String,
        password: String,
    },
}
//todo: handle edge cases like user sending prohibited chars (+, ], and  [ )
//todo: verbose error handling

pub enum TextDestination {
    Broadcast,
    Targets(Vec<String>),
}

impl TextDestination {
    pub fn from_raw(mut raw: String) -> Self {
        raw.pop();
        raw.remove(0);
        if raw == "BROADCAST" {
            TextDestination::Broadcast
        } else {
            TextDestination::Targets(raw.split(", ").into_iter().map(|v| v.to_owned()).collect())
        }
    }
}

impl Message {
    pub fn to_raw_message(self) -> String {
        match self {
            Message::Auth { id, password } => format!("[AUTH]+[{}@{}]", id, password),
            Message::Close => format!("[CLOSE]"),
            Message::Text {
                message,
                destinations: TextDestination::Broadcast,
            } => format!("[TEXT]+[BROADCAST]+[{}]", message),
            Message::Text {
                message,
                destinations: TextDestination::Targets(targets),
            } => format!("[TEXT]+[{}]+[{}]", targets.join(", "), message),
        }
    }

    pub fn from_raw_message(raw_message: String) -> Self {
        let chunks: Vec<&str> = raw_message.split('+').collect();
        let msg_type = chunks[0];
        match msg_type {
            "[CLOSE]" => Message::Close,
            "[AUTH]" => auth_creds(chunks[1].into()),
            _ => Message::Text {
                message: text_message_content(chunks[2].into()),
                destinations: TextDestination::from_raw(chunks[1].into()),
            },
        }
    }
}

fn text_message_content(mut raw: String) -> String {
    raw.pop();
    raw.remove(0);
    raw
}

fn auth_creds(mut creds: String) -> Message {
    creds.pop();
    creds.remove(0);
    let creds: Vec<&str> = creds.split("@").collect();
    Message::Auth {
        id: creds[0].into(),
        password: creds[1].into(),
    }
}
