use std::default::Default;

use protocol::ServerProtocol;
use conf::Config;
use cmd::IrcMsg;
use protocol::{ProtoErrorKind, ProtocolError};

/// This module implements Unreal protocol version 2311 (Unreal 3.2.10)

// TODO Review compile-flags sent to uplink
// TODO Change design to communicate "drop link"?

#[derive(Default)]
pub struct Unreal {
    synced: bool,
    nickv2: bool, // Use extended NICK message for introducing users.
    vhp: bool, // When introducing a user, send his cloaked host as if it were a vhost.
    umode2: bool, // Supports the UMODE2 command, a shortened version of MODE for usermode changes.
    vl: bool, // Supports V:Line information. Extends SERVER to include deny version{} blocks.
    sjoin: bool, // Supports SJOIN version 1 which is no longer in use. Use with SJ3.
    sjoin2: bool, // Supports SJOIN version 2 which is no longer in use. Use with SJ3.
    sj3: bool, // Supports SJOIN version 3.
    tkl: bool, // Supports exntended TKL messages for spamfilter support.
    nickip: bool, // Adds a base64 encoding of the user's ip in the NICK message parameters.
    clk: bool, // Supports an extra field in NICK for sending the cloaked host (not vhost).
}

impl ServerProtocol for Unreal {

    type IRCd = Unreal;

    fn new() -> Self {
        Unreal { synced: false, ..Default::default() }
    }

    /// Generates the introduce msg to an Unreal uplink.
    fn introduce_msg(&self, config: &Config) -> String {
        format!(concat!("PASS :{}\r\n",
                        "PROTOCTL NICKv2 VHP UMODE2 VL SJOIN SJOIN2 SJ3 TKLEXT NICKIP CLK\r\n",
                        "SERVER {} 1 :U2311-OoE-{} {}\r\n"),
                        config.get_link_passwd(),
                        config.get_server_name(), config.get_numeric(), config.get_description())
    }

    fn handle_pass(&self, config: &Config, msg: &IrcMsg) -> Result<Option<String>, ProtocolError> {
        if self.synced {
            Err(ProtocolError::new(ProtoErrorKind::InvalidContext,
                                   "Got PASS on an already-established link",
                                   None))
        } else if msg.params.len() == 0 {
            Err(ProtocolError::new(ProtoErrorKind::Fatal,
                                   "Empty PASS command",
                                   None))
        } else if &msg.params[0][..] != config.get_passwd_receive() {
            Err(ProtocolError::new(ProtoErrorKind::Fatal,
                                   "Wrong password received",
                                   Some(format!("PASS :{}", &msg.params[0][..]))))
        } else {
            Ok(None)
        }
    }

    fn handle_generic(&mut self, config: &Config, msg: &IrcMsg) ->
        Result<Option<String>, ProtocolError> {
            match &msg.command[..] {
                "PROTOCTL" => self.handle_protoctl(config, msg),
                _ => Ok(None)
            }
        }
}

impl Unreal {
    fn handle_protoctl(&mut self, config: &Config, msg: &IrcMsg) ->
        Result<Option<String>, ProtocolError> {
            if self.synced {
                return Err(ProtocolError::new(ProtoErrorKind::InvalidContext,
                                              "Got PROTOCTL on an already-established link",
                                              None));
            }
            if msg.params.len() == 0 {
                return Err(ProtocolError::new(ProtoErrorKind::MissingParameter,
                                              "Empty PROTOCTL command",
                                              None));
            }
            for token in msg.params.iter() {
                match &token[..] {
                    "NICKv2" => self.nickv2 = true,
                    "VHP" => self.vhp = true,
                    "UMODE2" => self.umode2 = true,
                    "VL" => self.vl = true,
                    "SJOIN" => self.sjoin = true,
                    "SJOIN2" => self.sjoin2 = true,
                    "SJ3" => self.sj3 = true,
                    "TKL" => self.tkl = true,
                    "NICKIP" => self.nickip = true,
                    "CLK" => self.clk = true,
                    _ => ()
                }
            }
            Ok(None)
    }
}
