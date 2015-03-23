use std::default::Default;

use protocol::ServerProtocol;
use conf::Config;
use cmd::IrcMsg;
use protocol::{ProtoErrorKind, ProtocolError};

/// This module implements Unreal protocol version 2311 (Unreal 3.2.10)

// TODO Review compile-flags sent to uplink
// TODO Change design to communicate "drop link"?

static PROTOVERSION: &'static str = "U2311";
static COMPILEFLAGS: &'static str = "Ooe";

#[derive(Default)]
pub struct Unreal {
    synced: bool,
    vhp: bool, // When introducing a user, send his cloaked host as if it were a vhost.
    umode2: bool, // Supports the UMODE2 command, a shortened version of MODE for usermode changes.
    vl: bool, // Supports V:Line information. Extends SERVER to include deny version{} blocks.
    sjoin: bool, // Supports SJOIN version 1 which is no longer in use. Use with SJ3.
    sjoin2: bool, // Supports SJOIN version 2 which is no longer in use. Use with SJ3.
    sj3: bool, // Supports SJOIN version 3.
    tkl: bool, // Supports exntended TKL messages for spamfilter support.
}

impl ServerProtocol for Unreal {

    type IRCd = Unreal;

    fn new() -> Self {
        Unreal { synced: false, ..Default::default() }
    }

    /// Generates the introduce msg to an Unreal uplink.
    fn introduce_msg(&self, config: &Config) -> String {
        format!(concat!("PASS :{}\r\n",
                        "PROTOCTL VHP UMODE2 VL SJOIN SJOIN2 SJ3 TKLEXT\r\n",
                        "SERVER {} 1 :{}-{}-{} {}\r\n"),
                config.get_link_passwd(), config.get_server_name(), PROTOVERSION, COMPILEFLAGS,
                config.get_numeric(), config.get_description())
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

    fn handle_server(&self, config: &Config, msg: &IrcMsg) ->
        Result<Option<String>, ProtocolError> {
            /* Unreal uses empty prefixes to introduce the uplink, and non-empty prefixes to
             * introduce servers with hopcount > 1
             * SERVER Ping.MindForge.org 1 :U2311-Fhin6XeOoEm-191 Ping? Pong!
             * :Ping.MindForge.org SMO o :(link) Link Ping.MindForge.org -> RustPower.MindForge.org[@0:0:0:0:0:ffff:85.241.8.245.60416] established
             * :Ping.MindForge.org SERVER SanFrancisco.MindForge.org 2 :Oh, California!
             * :SanFrancisco.MindForge.org EOS
             */
            if msg.source.is_some() {
                // We don't care about other servers :)
                return Ok(None);
            }

            if msg.params.len() < 3 {
                return Err(ProtocolError::new(ProtoErrorKind::Fatal,
                                              "Invalid SERVER message (missing parameters)",
                                              None));
            }

            if &msg.params[0][..] != config.get_uplink_name() {
                return Err(ProtocolError::new(ProtoErrorKind::Fatal,
                                              "Wrong uplink server name",
                                              Some(format!("Got {}, expected {}",
                                                           &msg.params[0][..],
                                                           config.get_uplink_name()))));
            }

            if !&msg.params[2][..].starts_with(PROTOVERSION) {
                return Err(ProtocolError::new(ProtoErrorKind::ProtocolVMismatch,
                                              "Different protocol version",
                                              Some(format!("Uplink implements {}, we implement {}",
                                                           &msg.params[2][..], PROTOVERSION))));
            }

            Ok(None)
        }

    fn handle_generic(&mut self, config: &Config, msg: &IrcMsg) ->
        Result<Option<String>, ProtocolError> {
            match &msg.command[..] {
                "PROTOCTL" => self.handle_protoctl(msg),
                "EOS" => self.handle_eos(config, msg),
                _ => Ok(None)
            }
        }
}

impl Unreal {
    fn handle_protoctl(&mut self, msg: &IrcMsg) ->
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
                    "VHP" => self.vhp = true,
                    "UMODE2" => self.umode2 = true,
                    "VL" => self.vl = true,
                    "SJOIN" => self.sjoin = true,
                    "SJOIN2" => self.sjoin2 = true,
                    "SJ3" => self.sj3 = true,
                    "TKL" => self.tkl = true,
                    _ => ()
                }
            }
            Ok(None)
    }

    fn handle_eos(&mut self, config: &Config, msg: &IrcMsg) ->
        Result<Option<String>, ProtocolError> {
        let uname = config.get_uplink_name();
        if msg.source.as_ref().map_or(uname, |p| &p[..]) == uname {
            if self.synced {
                Err(ProtocolError::new(ProtoErrorKind::InvalidContext,
                                       "GOT EOS on an already-established link",
                                       None))
            } else {
                self.synced = true;
                Ok(Some(format!("EOS\r\n")))
            }
        } else {
            Ok(None)
        }
    }
}
