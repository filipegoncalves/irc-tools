
mod conn;

use irc::conn::NetStream;
use cmd::{IrcMsg, IrcMessage};
use protocol::ServerProtocol;
use protocol::ProtoErrorKind;
use conf::Config;

use encoding::{DecoderTrap, EncoderTrap, Encoding};
use encoding::label::encoding_from_whatwg_label;

#[cfg(feature = "ssl")] use openssl::ssl::{SslStream, SslMethod, SslContext};
#[cfg(feature = "ssl")] use openssl::ssl::error::SslError;
#[cfg(feature = "ssl")] use std::result::Result as StdResult;

use std::io::{BufStream, BufRead, Result, Write};
use std::io::Error as IoError;
use std::io::ErrorKind;
use std::net::TcpStream;
use std::str::FromStr;
use std::error::Error;
use std::borrow::ToOwned;
use std::rc::Rc;
use std::cell::RefCell;

pub struct IrcStream<T: ServerProtocol> {
    stream: Rc<RefCell<BufStream<NetStream>>>,
    protocol_handler: RefCell<T>,
    config: Rc<RefCell<Config>>
}

pub struct IrcStreamIterator<'a, T: 'a + ServerProtocol> {
    ircstream: &'a IrcStream<T>
}

impl<'a, T: 'a + ServerProtocol> IrcStream<T> {
    pub fn new(conf: Rc<RefCell<Config>>, phandler: T) -> Result<IrcStream<T>> {
        let ssl = conf.borrow().use_ssl();
        if ssl {
            IrcStream::new_ssl_stream(conf, phandler)
        } else {
            IrcStream::new_plain_stream(conf, phandler)
        }
    }

    pub fn iter(&'a self) -> IrcStreamIterator<'a, T> {
        IrcStreamIterator::new(self)
    }

    #[cfg(feature = "ssl")]
    fn new_ssl_stream(conf: Rc<RefCell<Config>>, phandler: T) -> Result<IrcStream<T>> {
        let socket = try!(TcpStream::connect(&format!("{}:{}",
                                                      conf.borrow().get_uplink_addr(),
                                                      conf.borrow().get_uplink_port())[..]));
        let ssl_ctx = try!(ssl_to_io(SslContext::new(SslMethod::Tlsv1)));
        let ssl_socket = try!(ssl_to_io(SslStream::new(&ssl_ctx, socket)));

        Ok(IrcStream {
            stream: Rc::new(RefCell::new(BufStream::new(NetStream::SecureNetStream(ssl_socket)))),
            protocol_handler: RefCell::new(phandler),
            config: conf })
    }

    #[cfg(not(feature = "ssl"))]
    fn new_ssl_stream(conf: Rc<RefCell<Config>>, phandler: T) -> Result<IrcStream<T>> {
        panic!("SSL support was not compiled, but use_ssl is set to 'yes'. Please recompile with ssl support by enabling the feature 'ssl'");
    }

    fn new_plain_stream(conf: Rc<RefCell<Config>>, phandler: T) -> Result<IrcStream<T>> {
        let socket = NetStream::PlainNetStream(try!(TcpStream::connect(
            &format!("{}:{}",
                     conf.borrow().get_uplink_addr(),
                     conf.borrow().get_uplink_port())[..])));

        Ok(IrcStream {
            stream: Rc::new(RefCell::new(BufStream::new(socket))),
            protocol_handler: RefCell::new(phandler),
            config: conf })
    }

    pub fn introduce(&self) -> Result<()> {
        let intro_msg = &self.protocol_handler.borrow().introduce_msg()[..];
        self.send_msg(intro_msg)
    }

    pub fn recv_msg(&self) -> Result<IrcMessage> {
        let mut line = String::new();
        self.read_line(&mut line).and_then(|_| {
            let msg = IrcMsg::from_str(&line[..]);
            if msg.is_err() {
                return Ok(msg);
            }
            match self.protocol_handler.borrow_mut().handle(msg.as_ref().unwrap()) {
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
        
    pub fn send_msg(&self, msg: &str) -> Result<()> {
        self.write_line(msg)
    }

    fn write_line(&self, msg: &str) -> Result<()> {
        let borrowed = self.config.borrow();
        let charset = borrowed.get_encoding();
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

        try!(self.stream.borrow_mut().write_all(&data));
        self.stream.borrow_mut().flush().and_then(
            |_| { print!("[RAW OUTPUT]: {}", msg); Ok::<(), _>(()) })
    }

    // TODO Proper logging
    fn read_line(&self, buff: &mut String) -> Result<()> {
        let borrowed = self.config.borrow();
        let charset = borrowed.get_encoding();
        let encoding = match encoding_from_whatwg_label(charset) {
            Some(enc) => enc,
            None => return Err(IoError::new(ErrorKind::InvalidInput, "Failed to find decoder.",
                                            Some(format!("Invalid decoder: {}", charset))))
        };

        let mut buf = Vec::new();
        self.stream.borrow_mut().read_until(b'\n', &mut buf).and_then(|_| {
            match encoding.decode(&buf, DecoderTrap::Replace) {
                Ok(data) => { *buff = data; print!("[RAW INPUT]: {}", buff); Ok::<(), _>(()) },
                Err(e) => Err(IoError::new(ErrorKind::InvalidInput, "Failed to decode message.",
                                           Some(format!("Failed to decode {} as {}.", e,
                                                        encoding.name()))))
            }
        })
    }
}

impl<'a, T: 'a + ServerProtocol> IrcStreamIterator<'a, T> {
    pub fn new(istream: &'a IrcStream<T>) -> IrcStreamIterator<T> {
        IrcStreamIterator { ircstream: istream }
    }
}

impl<'a, T: ServerProtocol + 'a> Iterator for IrcStreamIterator<'a, T> {
    type Item = Result<IrcMessage>;

    fn next(&mut self) -> Option<Result<IrcMessage>> {
        Some(self.ircstream.recv_msg())
    }
}

/// Converts a Result<U, SslError> into a Result<U>.
#[cfg(feature = "ssl")]
fn ssl_to_io<U>(res: StdResult<U, SslError>) -> Result<U> {
    match res {
        Ok(x) => Ok(x),
        Err(e) => Err(IoError::new(ErrorKind::Other, "An SSL error occurred.",
                                   Some(e.description().to_owned()))),
    }
}
