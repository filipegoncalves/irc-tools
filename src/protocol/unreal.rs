use protocol::ServerProtocol;

pub struct Unreal;

impl ServerProtocol for Unreal {

    type IRCd = Unreal;

    fn introduce_msg(&self, passwd: &str, name: &str, numeric: u16, desc: &str) -> String {
        format!("PASS :{}\r\nSERVER {} 1 {} :{}\r\n", passwd, name, numeric, desc)
    }
}
