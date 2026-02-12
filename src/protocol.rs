
pub enum Message {
    Close,
    Text {
        message: String,
        destinations: TextDestination
    },
    Auth {
        password: String
    }
}
//todo: handle edge cases like user sending prohibited chars (+, ], and  [ )
//todo: verbose error handling

pub enum TextDestination {
    Broadcast,
    Targets(Vec<String>)
}

impl TextDestination {
   pub fn from_raw(mut raw: String) -> Self {
    raw.pop();
    raw.remove(0);
    if raw == "BROADCAST" {
        TextDestination::Broadcast
    }else{
        TextDestination::Targets(raw.split(", ").into_iter().map(|v|v.to_owned()).collect())
    }
   }
}

impl Message {
    pub fn to_raw_message(self) -> String {
        match self {
            Message::Auth { password } => format!("[AUTH]+[{}]", password),
            Message::Close => format!("[CLOSE]"),
            Message::Text { message, destinations: TextDestination::Broadcast }  => format!("[TEXT]+[BROADCAST]+[{}]", message),
            Message::Text { message, destinations: TextDestination::Targets(targets) } => format!("[TEXT]+[{}]+[{}]", targets.join(", "),message)
        }
    } 

    pub fn from_raw_message(raw_message: String) -> Self {
        let chunks: Vec<&str> = raw_message.split('+').collect();
        let msg_type = chunks[0];
        match msg_type {
            "[CLOSE]" => Message::Close,
            "[AUTH]" => Message::Auth { password: chunks[1].into() },
            _ => Message::Text { message: text_message_content(chunks[2].into()), destinations: TextDestination::from_raw(chunks[1].into()) }
        }
    }
}

fn text_message_content(mut raw: String)->String{
    raw.pop();
    raw.remove(0);
    raw
}