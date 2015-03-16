use std::borrow::ToOwned;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Error, ErrorKind};
use std::io::Result;
use std::io::Read;
use std::error::Error as StdError;
use rustc_serialize::json::decode;

/// Configuration data.
#[derive(RustcDecodable)]
pub struct Config {
    servname: String,
    numeric: u16,
    description: String,
    uplink: String,
    port: Option<u16>,
    password: String,
    use_ssl: bool,
    encoding: String,
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

    pub fn get_uplink_port(&self) -> u16 {
        self.port.unwrap_or(6667)
    }

    pub fn get_link_passwd(&self) -> &str {
        &self.password[..]
    }

    pub fn get_encoding(&self) -> &str {
        &self.encoding[..]
    }
}
