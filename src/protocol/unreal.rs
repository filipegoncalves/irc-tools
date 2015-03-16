use protocol::ServerProtocol;
use conf::Config;

pub struct Unreal;

impl ServerProtocol for Unreal {

    type IRCd = Unreal;

    fn introduce_msg(&self, config: &Config) -> String {
        format!("PASS :{}\r\nSERVER {} 1 {} :{}\r\n", config.get_link_passwd(),
                config.get_server_name(), config.get_numeric(), config.get_description())
    }
}
