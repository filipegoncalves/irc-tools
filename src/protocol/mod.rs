
pub mod unreal;

use cmd::IrcMsg;
use conf::Config;

use std::fmt::{Display, Formatter};
use std::fmt::Result as FmtResult;
use std::rc::Rc;
use std::cell::RefCell;

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
    /// Protocol version mismatch
    /// Example: Uplink runs UnrealIRCd with another protocol version
    ProtocolVMismatch,
    /// A fatal error that will cause the link to be terminated
    /// Example: Wrong link password / wrong server name
    Fatal
}

pub struct ProtocolError {
    pub kind: ProtoErrorKind,
    pub desc: &'static str,
    pub detail: Option<String>
}

pub enum IrcClientType {
    Regular,
    Service
}

pub trait ServerProtocol {

    type IRCd;

    fn new(config: Rc<RefCell<Config>>) -> Self;

    fn introduce_msg(&self) -> String;

    fn introduce_client_msg(&self, ctype: IrcClientType,
                            nick: &str, ident: &str, host: &str, gecos: &str) -> String;

    fn handle(&mut self, msg: &IrcMsg) -> Result<Option<String>, ProtocolError> {
        match &msg.command[..] {
            "PING" => self.handle_ping(msg),
            "PASS" => self.handle_pass(msg),
            _ => self.handle_generic(msg)
        }
    }

    fn handle_pass(&self, msg: &IrcMsg) -> Result<Option<String>, ProtocolError>;

    // TODO
    // When Rust supports struct inheritance, move handle_ping back here
    fn handle_ping(&self, msg: &IrcMsg) -> Result<Option<String>, ProtocolError>;

    fn handle_server(&self, msg: &IrcMsg) -> Result<Option<String>, ProtocolError>;

    #[allow(unused_variables)]
    fn handle_generic(&mut self, msg: &IrcMsg) ->
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
