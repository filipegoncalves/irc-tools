use std::default::Default;
use std::rc::Rc;
use std::cell::RefCell;

use protocol::ServerProtocol;
use conf::Config;
use cmd::IrcMsg;
use protocol::{ProtoErrorKind, ProtocolError};
use protocol::IrcClientType;

use time;

/// This module targets Unreal protocol version 2311 (Unreal 3.2.10)

// TODO Review compile-flags sent to uplink (think about sending SSL?)
// TODO NICKIP
// TODO Check nick against NICKCHARS in introduce_client()

static PROTOVERSION: &'static str = "U2311";
static COMPILEFLAGS: &'static str = "Ooe";
static DEF_USR_MODES: &'static str = "+i";
static DEF_SERVICE_MODES: &'static str = "+ioSq";

#[derive(Default)]
pub struct Unreal {
    /// Configuration
    conf: Rc<RefCell<Config>>,
    /// Are we synced?
    synced: bool,
    /// When introducing a user, send his cloaked host as if it were a vhost.
    vhp: bool,
    /// Supports the UMODE2 command, a shortened version of MODE for usermode changes.
    umode2: bool,
    /// Supports V:Line information. Extends SERVER to include deny version{} blocks.
    vl: bool,
    /// Supports SJOIN version 1 which is no longer in use. Use with SJ3.
    sjoin: bool,
    /// Supports SJOIN version 2 which is no longer in use. Use with SJ3.
    sjoin2: bool,
    /// Supports SJOIN version 3.
    sj3: bool,
    /// Supports exntended TKL messages for spamfilter support.
    tkl: bool,
    /// Use extended NICK message for introducing users.
    nickv2: bool,
    /// Adds an IP parameter to the NICK message, which is the base64 encoding of the user's
    /// ip address (in network byte order). Requires NICKv2.
    nickip: bool
}

impl ServerProtocol for Unreal {

    //type IRCd = Unreal;

    fn new(config: Rc<RefCell<Config>>) -> Self {
        Unreal { conf: config.clone(), synced: false, ..Default::default() }
    }

    /// Generates the introduce msg to an Unreal uplink.
    fn introduce_msg(&self) -> String {
        let conf = self.conf.borrow();
        format!(concat!("PASS :{}\r\n",
                        "PROTOCTL VHP UMODE2 VL SJOIN SJOIN2 SJ3 TKLEXT NICKv2 NICKIP\r\n",
                        "SERVER {} 1 :{}-{}-{} {}\r\n"),
                conf.get_link_passwd(), conf.get_server_name(), PROTOVERSION, COMPILEFLAGS,
                conf.get_numeric(), conf.get_description())
    }

    /// Generates a client introduce msg
    fn introduce_client_msg(&self, ctype: IrcClientType,
                            nick: &str, ident: &str, host: &str, gecos: &str) -> String {

        let conf = self.conf.borrow();

        let mut msg = format!("NICK {} 1 {} {} {} {} 0", nick, time::get_time().sec, ident, host,
                              conf.get_server_name());
        // TODO What if NICKv2 is not supported? We need to send modes anyway...
        // Same for NICKIP
        if self.nickv2 {
            let umodes = match ctype {
                IrcClientType::Regular => DEF_USR_MODES,
                IrcClientType::Service => DEF_SERVICE_MODES
            };
            msg.push_str(&format!(" {} {}", umodes, host)[..]);
            if self.nickip {
                // TODO Do not hardcode IP
                msg.push_str(" fwAAAQ==");
            }
        }

        msg.push_str(&format!(" :{}", gecos)[..]);

        msg
    }

    fn handle_pass(&self, msg: &IrcMsg) -> Result<Option<String>, ProtocolError> {
        if self.synced {
            Err(ProtocolError::new(ProtoErrorKind::InvalidContext,
                                   "Got PASS on an already-established link",
                                   None))
        } else if msg.params.len() == 0 {
            Err(ProtocolError::new(ProtoErrorKind::Fatal,
                                   "Empty PASS command",
                                   None))
        } else if &msg.params[0][..] != self.conf.borrow().get_passwd_receive() {
            Err(ProtocolError::new(ProtoErrorKind::Fatal,
                                   "Wrong password received",
                                   Some(format!("PASS :{}", &msg.params[0][..]))))
        } else {
            Ok(None)
        }
    }

    fn handle_ping(&self, msg: &IrcMsg) -> Result<Option<String>, ProtocolError> {
        let conf = self.conf.borrow();
        if msg.params.len() < 1 {
            return Err(ProtocolError::new(ProtoErrorKind::MissingParameter,
                                          "No parameters found; expected at least 1.",
                                          Some(format!("PING with no parameters"))));
        }
        if msg.params.len() >= 2 && &msg.params[1][..] != conf.get_server_name() {
            return Err(ProtocolError::new(ProtoErrorKind::InvalidParameter,
                                          "Request to act as a hub",
                                          Some(format!("PING {} :{}",
                                                       &msg.params[0][..], 
                                                       &msg.params[1][..]))));
        }
        if msg.params[0] != conf.get_uplink_name() {
            Ok(Some(format!("PONG {} :{}\r\n", conf.get_server_name(), &msg.params[0][..])))
        } else {
            Ok(Some(format!("PONG :{}\r\n", conf.get_server_name())))
        }
    }


    fn handle_server(&self, msg: &IrcMsg) ->
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

            if &msg.params[0][..] != self.conf.borrow().get_uplink_name() {
                return Err(ProtocolError::new(ProtoErrorKind::Fatal,
                                              "Wrong uplink server name",
                                              Some(format!("Got {}, expected {}",
                                                           &msg.params[0][..],
                                                           self.conf.borrow().get_uplink_name()))));
            }

            if !&msg.params[2][..].starts_with(PROTOVERSION) {
                return Err(ProtocolError::new(ProtoErrorKind::ProtocolVMismatch,
                                              "Different protocol version",
                                              Some(format!("Uplink implements {}, we implement {}",
                                                           &msg.params[2][..], PROTOVERSION))));
            }

            Ok(None)
        }

    fn handle_generic(&mut self, msg: &IrcMsg) ->
        Result<Option<String>, ProtocolError> {
            match &msg.command[..] {
                "PROTOCTL" => self.handle_protoctl(msg),
                "EOS" => self.handle_eos(msg),
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
                    "NICKv2" => self.nickv2 = true,
                    "NICKIP" => self.nickip = true,
                    _ => ()
                }
            }
            Ok(None)
    }

    fn handle_eos(&mut self, msg: &IrcMsg) ->
        Result<Option<String>, ProtocolError> {
        let conf = self.conf.borrow();
        let uname = conf.get_uplink_name();
        if msg.source.as_ref().map_or(uname, |p| &p[..]) == uname {
            if self.synced {
                Err(ProtocolError::new(ProtoErrorKind::InvalidContext,
                                       "GOT EOS on an already-established link",
                                       None))
            } else {
                self.synced = true;
                // TODO Some sort of OnSync()
                let mut cbot_intro = self.introduce_client_msg(IrcClientType::Service,
                                                               conf.get_cbot_nick(),
                                                               conf.get_cbot_ident(),
                                                               conf.get_cbot_host(),
                                                               conf.get_cbot_gecos());

                cbot_intro.push_str("\r\n");

                for chan in conf.get_cbot_chans() {
                    cbot_intro.push_str(&format!(":{} JOIN {}\r\n", conf.get_cbot_nick(), chan))
                }

                Ok(Some(format!("{}EOS\r\n", cbot_intro)))
            }
        } else {
            Ok(None)
        }
    }
}
