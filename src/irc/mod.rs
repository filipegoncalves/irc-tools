
mod conn;

use irc::conn::NetStream;
use cmd::{IrcMsg, IrcMessage};
use protocol::ServerProtocol;

use encoding::{DecoderTrap, EncoderTrap, Encoding};
use encoding::label::encoding_from_whatwg_label;

use std::io::{BufStream, BufRead, Result, Write};
use std::net::TcpStream;
use std::str::FromStr;
use std::io::Error as IoError;
use std::io::ErrorKind;

pub struct IrcStream<T: ServerProtocol> {
    stream: BufStream<NetStream>,
    protocol_handler: T
}

impl<T: ServerProtocol> IrcStream<T> {
    pub fn new(server: &str, port: u16, phandler: T) -> Result<IrcStream<T>> {
        let socket = NetStream::PlainNetStream(try!(TcpStream::connect(&format!("{}:{}", server, port)[..])));
        Ok(IrcStream { stream: BufStream::new(socket), protocol_handler: phandler })
    }

    pub fn introduce(&mut self, pass: &str, name: &str, numeric: u16, desc: &str) -> Result<()> {
        let intro_msg = &self.protocol_handler.introduce_msg(pass, name, numeric, desc)[..];
        self.send_msg(intro_msg)
    }

    pub fn recv_msg(&mut self, encoding: &str) -> Result<IrcMessage> {
        let mut line = String::new();
        //self.read_line(&mut line).and_then(|_| Ok(IrcMsg::from_str(&line[..])))
        self.read_line(encoding, &mut line).and_then(|_| {
            let msg = IrcMsg::from_str(&line[..]);
            if msg.is_err() {
                return Ok(msg);
            }
            if let Some(reply) = self.protocol_handler.handle_internal(msg.as_ref().unwrap()) {
                self.send_msg(&reply[..]).and_then(|_| Ok(msg))
            } else {
                Ok(msg)
            }
        })
    }
        
    pub fn send_msg(&mut self, msg: &str) -> Result<()> {
        self.write_line(msg)
    }

    // TODO Proper logging
    fn read_line(&mut self, encoding: &str, buff: &mut String) -> Result<()> {
        let encoding = match encoding_from_whatwg_label(encoding) {
            Some(enc) => enc,
            None => return Err(IoError::new(ErrorKind::InvalidInput, "Failed to find decoder.",
                                            Some(format!("Invalid decoder: {}", encoding))))
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

    // TODO Proper logging
    fn write_line(&mut self, msg: &str) -> Result<()> {
        try!(self.stream.write_all(msg.as_bytes()));
        self.stream.flush().and_then(|_| { print!("[RAW OUTPUT]: {}", msg); Ok::<(), _>(()) })
    }
}
