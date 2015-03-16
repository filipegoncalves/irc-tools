use std::str::FromStr;
use std::borrow::ToOwned;

pub struct IrcMsg {
    pub source: Option<String>,
    pub command: String,
    pub params: Vec<String>
}

pub type IrcMessage = Result<IrcMsg, String>;

impl IrcMsg {
    fn new(src: Option<String>, cmd: &str, p: Vec<String>) -> IrcMsg {
        IrcMsg { source: src, command: cmd.to_owned(), params: p }
    }
}

impl FromStr for IrcMsg {
    type Err = String;

    fn from_str(m: &str) -> IrcMessage {
        let mut to_process = m.clone();

        if let Some(end) = to_process.find("\r\n") {
            to_process = &to_process[..end];
        } else {
            return Err("CR LF terminators not found.".to_string());
        }

        if to_process.len() == 0 {
            return Err("Empty message.".to_string());
        }

        // We start with the last param because it's the easiest to identify
        // and then we can remove it from to_process
        let last_param = if let Some(beg) = to_process.find(" :") {
            let l = &to_process[beg+2..];
            to_process = &to_process[..beg];
            l
        } else {
            ""
        };

        let pref = if m.starts_with(":") {
            if let Some(end) = m.find(' ') {
                let p = Some(format!("{}", &m[1..end]));
                to_process = &to_process[end+1..];
                p
            } else {
                return Err("Prefix found, but there's no space separator.".to_string());
            }
        } else {
            None
        };

        let command = if let Some(end) = to_process.find(' ') {
            let c = &to_process[..end];
            to_process = &to_process[end+1..];
            c
        } else {
            let c = &to_process[..];
            to_process = "";
            c
        };

        let mut params: Vec<_> = to_process.splitn(14, ' ').filter_map(
            |s| { if s.len() == 0 { None } else { Some(s.to_owned()) } }).collect();

        if last_param.len() > 0 {
            params.push(last_param.to_owned());
        }

        if command.len() > 0 {
            Ok(IrcMsg::new(pref, command, params))
        } else {
            Err("Empty command.".to_string())
        }

    }
}

#[cfg(test)]
mod test {
    use super::IrcMsg;
    use std::str::FromStr;
    #[test]
    fn ping() {
        let ping1 = ":services.MindForge.org PING services.MindForge.org :RustPower.MindForge.org\r\n";
        let ping2 = "PING :Ping.MindForge.org\r\n";
        let ping3 = "PING\r\n";
        let ping4 = ":services.MindForge.org PING \r\n";
        let ping5 = "PING server1.example.com server2.example.com server3.example.com :test.com\r\n";

        let mut res = IrcMsg::from_str(ping1);
        let mut msg;
        assert!(res.is_ok());
        msg = res.unwrap();
        assert!(msg.source.is_some());
        assert!(msg.source.unwrap() == "services.MindForge.org");
        assert!(msg.command == "PING");
        assert!(msg.params.len() == 2);
        assert!(msg.params[0] == "services.MindForge.org");
        assert!(msg.params[1] == "RustPower.MindForge.org");

        res = IrcMsg::from_str(ping2);
        assert!(res.is_ok());
        msg = res.unwrap();
        assert!(msg.source.is_none());
        assert!(msg.command == "PING");
        assert!(msg.params.len() == 1);
        assert!(msg.params[0] == "Ping.MindForge.org");

        res = IrcMsg::from_str(ping3);
        assert!(res.is_ok());
        msg = res.unwrap();
        assert!(msg.source.is_none());
        assert!(msg.command == "PING");
        assert!(msg.params.len() == 0);

        res = IrcMsg::from_str(ping4);
        assert!(res.is_ok());
        msg = res.unwrap();
        assert!(msg.source.is_some());
        assert!(msg.source.unwrap() == "services.MindForge.org");
        assert!(msg.command == "PING");
        assert!(msg.params.len() == 0);

        res = IrcMsg::from_str(ping5);
        assert!(res.is_ok());
        msg = res.unwrap();
        assert!(msg.source.is_none());
        assert!(msg.command == "PING");
        assert!(msg.params.len() == 4);
        assert!(msg.params[3] == "test.com");
        assert!(msg.params[0] == "server1.example.com");
        assert!(msg.params[1] == "server2.example.com");
        assert!(msg.params[2] == "server3.example.com");
    }
}
