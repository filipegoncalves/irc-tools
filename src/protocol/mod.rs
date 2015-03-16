
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

    fn handle_ping(&self, config: &Config, msg: &IrcMsg) -> Option<String> {
        if msg.params.len() < 1 {
            println!("Invalid PING, missing parameters.");
            return None;
        }
        if msg.params.len() >= 2 && &msg.params[1][..] != config.get_server_name() {
            println!("Invalid PING, I am not a hub: PING {} :{}",
                     &msg.params[0][..], &msg.params[1][..]);
            return None;
        }
        let mut reply = format!("PONG {}", config.get_server_name());
        if msg.params[0] != config.get_uplink_name() {
            reply.push_str(" :");
            reply.push_str(&msg.params[0]);
        }
        reply.push_str("\r\n");
        Some(reply)
    }

}
