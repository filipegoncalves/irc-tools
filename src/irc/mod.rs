
mod conn;

use irc::conn::NetStream;
use cmd::{IrcMsg, IrcMessage};
use protocol::ServerProtocol;
use protocol::ProtoErrorKind;
use conf::Config;

use encoding::{DecoderTrap, EncoderTrap, Encoding};
use encoding::label::encoding_from_whatwg_label;

use std::io::{BufStream, BufRead, Result, Write};
use std::net::TcpStream;
use std::str::FromStr;
use std::io::Error as IoError;
use std::io::ErrorKind;

pub struct IrcStream<T: ServerProtocol> {
    stream: BufStream<NetStream>,
    protocol_handler: T,
    config: Config
}

impl<T: ServerProtocol> IrcStream<T> {
    pub fn new(conf: Config, phandler: T) -> Result<IrcStream<T>> {
        let socket = NetStream::PlainNetStream(try!(TcpStream::connect(
            &format!("{}:{}", conf.get_uplink_addr(), conf.get_uplink_port())[..])));

        Ok(IrcStream { stream: BufStream::new(socket), protocol_handler: phandler, config: conf })
    }

    pub fn introduce(&mut self) -> Result<()> {
        let intro_msg = &self.protocol_handler.introduce_msg(&self.config)[..];
        self.send_msg(intro_msg)
    }

    pub fn recv_msg(&mut self) -> Result<IrcMessage> {
        let mut line = String::new();
        //self.read_line(&mut line).and_then(|_| Ok(IrcMsg::from_str(&line[..])))
        self.read_line(&mut line).and_then(|_| {
            let msg = IrcMsg::from_str(&line[..]);
            if msg.is_err() {
                return Ok(msg);
            }
            match self.protocol_handler.handle(&self.config, msg.as_ref().unwrap()) {
                Ok(Some(reply)) => self.send_msg(&reply[..]).and_then(|_| Ok(msg)),
                Err(e)  => {
                    println!("{}", e);
                    if e.kind == ProtoErrorKind::Fatal {
                        Err(IoError::new(ErrorKind::InvalidInput, e.desc, e.detail))
                    } else {
                        Ok(msg)
                    }
                }
                _ => Ok(msg)
            }})
    }
        
    pub fn send_msg(&mut self, msg: &str) -> Result<()> {
        self.write_line(msg)
    }

    // TODO Proper logging
    fn read_line(&mut self, buff: &mut String) -> Result<()> {
        let charset = self.config.get_encoding();
        let encoding = match encoding_from_whatwg_label(charset) {
            Some(enc) => enc,
            None => return Err(IoError::new(ErrorKind::InvalidInput, "Failed to find decoder.",
                                            Some(format!("Invalid decoder: {}", charset))))
        };

        let mut buf = Vec::new();
        self.stream.read_until(b'\n', &mut buf).and_then(|_| {
            match encoding.decode(&buf, DecoderTrap::Replace) {
                Ok(data) => { *buff = data; print!("[RAW INPUT]: {}", buff); Ok::<(), _>(()) },
                Err(e) => Err(IoError::new(ErrorKind::InvalidInput, "Failed to decode message.",
                                           Some(format!("Failed to decode {} as {}.", e,
                                                        encoding.name()))))
            }
        })

        //self.stream.read_line(buff).and_then(
        //    |_| { print!("[RAW INPUT]: {}", buff); Ok::<(), _>(()) })
    }

    fn write_line(&mut self, msg: &str) -> Result<()> {
        let charset = self.config.get_encoding();
        let encoding = match encoding_from_whatwg_label(charset) {
            Some(enc) => enc,
            None => return Err(IoError::new(ErrorKind::InvalidInput, "Failed to find encoder.",
                                            Some(format!("Invalid encoder: {}", charset))))
        };

        let data = match encoding.encode(msg, EncoderTrap::Replace) {
            Ok(data) => data,
            Err(data) => return Err(IoError::new(ErrorKind::InvalidInput,
                                                 "Failed to encode message.",
                                                 Some(format!("Failed to encode {} as {}.",
                                                              data, encoding.name()))))
        };

        try!(self.stream.write_all(&data));
        self.stream.flush().and_then(|_| { print!("[RAW OUTPUT]: {}", msg); Ok::<(), _>(()) })
    }
}
