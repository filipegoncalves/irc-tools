
pub mod unreal;

use cmd::IrcMsg;
use conf::Config;

use std::fmt::{Display, Formatter};
use std::fmt::Result as FmtResult;

/// A list of possible error types in the server-to-server protocol
/// Any error that is not `Fatal` will yield a warning but keep the
/// link active. `Fatal` errors drop the connection to the server.
#[derive(Debug, PartialEq)]
pub enum ProtoErrorKind {
    /// Command is missing one or more required parameters
    /// Example: receiving PRIVMSG with 0 parameters
    MissingParameter,
    /// Invalid parameter value
    /// Example: receiving a PING that needs to be forwarded
    InvalidParameter,
    /// A command that cannot / was not expected in this context.
    /// Example: receiving PASS when the link is already established.
    InvalidContext,
    /// A fatal error that will cause the link to be terminated
    /// Example: Wrong link password / wrong server name
    Fatal
}

pub struct ProtocolError {
    pub kind: ProtoErrorKind,
    pub desc: &'static str,
    pub detail: Option<String>
}

pub trait ServerProtocol {

    type IRCd;

    fn new() -> Self;

    fn introduce_msg(&self, config: &Config) -> String;

    fn handle(&mut self, config: &Config, msg: &IrcMsg) -> Result<Option<String>, ProtocolError> {
        match &msg.command[..] {
            "PING" => self.handle_ping(config, msg),
            "PASS" => self.handle_pass(config, msg),
            _ => self.handle_generic(config, msg)
        }
    }

    fn handle_pass(&self, config: &Config, msg: &IrcMsg) -> Result<Option<String>, ProtocolError>;

    fn handle_ping(&self, config: &Config, msg: &IrcMsg) -> Result<Option<String>, ProtocolError> {
        if msg.params.len() < 1 {
            return Err(ProtocolError::new(ProtoErrorKind::MissingParameter,
                                          "No parameters found; expected at least 1.",
                                          Some(format!("PING with no parameters"))));
        }
        if msg.params.len() >= 2 && &msg.params[1][..] != config.get_server_name() {
            return Err(ProtocolError::new(ProtoErrorKind::InvalidParameter,
                                          "Request to act as a hub",
                                          Some(format!("PING {} :{}",
                                                       &msg.params[0][..], 
                                                       &msg.params[1][..]))));
        }
        if msg.params[0] != config.get_uplink_name() {
            Ok(Some(format!("PONG {} :{}\r\n", config.get_server_name(), &msg.params[0][..])))
        } else {
            Ok(Some(format!("PONG :{}\r\n", config.get_server_name())))
        }
    }

    #[allow(unused_variables)]
    fn handle_generic(&mut self, config: &Config, msg: &IrcMsg) ->
        Result<Option<String>, ProtocolError> {
        Ok(None)
    }
}

impl ProtocolError {
    fn new(errtype: ProtoErrorKind, descr: &'static str, details: Option<String>) -> ProtocolError {
        ProtocolError { kind: errtype, desc: descr, detail: details }
    }
}

impl Display for ProtocolError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "[PROTOCOL ERROR] ({:?}): {} ({})",
               self.kind,
               self.desc,
               self.detail.as_ref().map_or("no details", |d| &d[..]))
    }
}
