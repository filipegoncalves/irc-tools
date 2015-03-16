
pub mod unreal;

use cmd::IrcMsg;
use conf::Config;

pub trait ServerProtocol {

    type IRCd;

    fn introduce_msg(&self, config: &Config) -> String;

    fn handle(&self, config: &Config, msg: &IrcMsg) -> Option<String> {
        match &msg.command[..] {
            "PING" => self.handle_ping(config, msg),
            _ => None
        }
    }

    // TODO Hardcoded server names
    fn handle_ping(&self, config: &Config, msg: &IrcMsg) -> Option<String> {
        if msg.params.len() < 1 {
            println!("Invalid PING command, missing parameters.");
            return None;
        }
        if msg.params.len() >= 2 && &msg.params[1][..] != "RustPower.MindForge.org" {
            println!("Invalid PING command, I am not a hub.");
            return None;
        }
        let mut reply = "PONG RustPower.MindForge.org".to_string();
        if msg.params[0] != "Ping.MindForge.org" {
            reply.push_str(" :");
            reply.push_str(&msg.params[0]);
        }
        reply.push_str("\r\n");
        Some(reply)
    }

}
