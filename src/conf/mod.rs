use std::borrow::{Borrow, ToOwned};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Error, ErrorKind};
use std::io::Result;
use std::io::Read;
use std::error::Error as StdError;
use std::path::Path;
use rustc_serialize::json::decode;

/// Configuration data.
#[derive(RustcDecodable)]
pub struct Config {
    servname: String,
    numeric: u16,
    description: String,
    uplink: String,
    uplinkname: String,
    port: Option<u16>,
    password: String,
    pass_receive: String,
    use_ssl: bool,
    encoding: String,
    cbot_nick: String,
    cbot_ident: String,
    cbot_host: String,
    cbot_gecos: String,
    cbot_chans: Vec<String>,
    options: HashMap<String, String>
}

#[stable]
impl Config {
    /// Loads a JSON configuration from the desired path.
    #[stable]
    pub fn load(path: &Path) -> Result<Config> {
        let mut file = try!(File::open(path));
        let mut data = String::new();
        try!(file.read_to_string(&mut data));
        decode(&data[..]).map_err(|e| Error::new(ErrorKind::InvalidInput,
                                                 "Failed to decode configuration file.",
                                                 Some(e.description().to_owned())))
    }

    pub fn get_server_name(&self) -> &str {
        &self.servname[..]
    }

    pub fn get_numeric(&self) -> u16 {
        self.numeric
    }

    pub fn get_description(&self) -> &str {
        &self.description[..]
    }

    pub fn get_uplink_addr(&self) -> &str {
        &self.uplink[..]
    }

    pub fn get_uplink_name(&self) -> &str {
        &self.uplinkname[..]
    }

    #[cfg(not(feature = "ssl"))]
    pub fn get_uplink_port(&self) -> u16 {
        self.port.unwrap_or(6667)
    }

    #[cfg(feature = "ssl")]
    pub fn get_uplink_port(&self) -> u16 {
        self.port.unwrap_or(6697)
    }

    pub fn get_link_passwd(&self) -> &str {
        &self.password[..]
    }

    pub fn get_passwd_receive(&self) -> &str {
        &self.pass_receive[..]
    }

    pub fn get_encoding(&self) -> &str {
        &self.encoding[..]
    }

    pub fn use_ssl(&self) -> bool {
        self.use_ssl
    }

    pub fn get_cbot_nick(&self) -> &str {
        &self.cbot_nick[..]
    }

    pub fn get_cbot_ident(&self) -> &str {
        &self.cbot_ident[..]
    }

    pub fn get_cbot_host(&self) -> &str {
        &self.cbot_host[..]
    }

    pub fn get_cbot_gecos(&self) -> &str {
        &self.cbot_gecos[..]
    }

    pub fn get_cbot_chans(&self) -> &[String] {
        self.cbot_chans.borrow()
    }
    //pub fn get_option(&self) -> Option<
}
