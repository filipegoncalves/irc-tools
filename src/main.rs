#![feature(io)]
#![feature(net)]
#![feature(fs)]
#![feature(core)]
#![feature(old_path)]

extern crate "rustc-serialize" as rustc_serialize;
extern crate encoding;

mod irc;
mod cmd;
mod conf;
mod protocol;

use irc::IrcStream;
use conf::Config;
use std::io::Result;

use protocol::unreal::Unreal;
use protocol::ServerProtocol;


/*
[RAW INPUT]: PASS :rustp0w3r!
[RAW INPUT]: PROTOCTL NOQUIT TOKEN NICKv2 SJOIN SJOIN2 UMODE2 VL SJ3 NS SJB64 TKLEXT NICKIP ESVID
[RAW INPUT]: PROTOCTL CHANMODES=beI,kfL,lj,psmntirRcOAQKVCuzNSMTGZ NICKCHARS= MLOCK
[RAW INPUT]: SERVER Ping.MindForge.org 1 :U2311-Fhin6XeOoEm-191 Ping? Pong!
[RAW INPUT]: :Ping.MindForge.org SMO o :(link) Link Ping.MindForge.org -> RustPower.MindForge.org[@0:0:0:0:0:ffff:85.241.8.245.60416] established
[RAW INPUT]: :Ping.MindForge.org SERVER SanFrancisco.MindForge.org 2 :Oh, California!
[RAW INPUT]: :SanFrancisco.MindForge.org EOS
[RAW INPUT]: :Ping.MindForge.org SERVER tools.MindForge.org 2 :MindForge Tools
[RAW INPUT]: :tools.MindForge.org EOS
[RAW INPUT]: :Ping.MindForge.org SERVER Ocean.MindForge.org 2 :Kitties can't swim
[RAW INPUT]: :Ocean.MindForge.org EOS
[RAW INPUT]: :Ping.MindForge.org SERVER services.MindForge.org 1 :MindForge IRC Services
[RAW INPUT]: :services.MindForge.org EOS
[RAW INPUT]: NICK eMuleChansDrop 2 1425754439 eMule Bot.MindForge.org tools.MindForge.org MFTooL :eMule Chans Auto Drop
[RAW INPUT]: :eMuleChansDrop MODE eMuleChansDrop :+iorSq
[RAW INPUT]: :eMuleChansDrop JOIN #ServicesLog
[RAW INPUT]: NICK MFTooL 2 1425754439 TooL MindForge.org tools.MindForge.org MFTooL :MindForge Security Tool
*/


//TODO deal with case-sensitiveness
// TODO Encoding

fn main() {

    // TODO make cmd-line option to pass conf file path
    // TODO Do not handle errors silently

    let config = match load_config("tools.conf") {
        Ok(cfg) => cfg,
        Err(e) => { println!("conf error: {}", e.description()); return () }
    };

    let mut ircstream = match IrcStream::new(config.get_uplink_addr(), config.get_uplink_port(),
                                             Unreal) {
        Ok(stream) => stream,
        Err(_) => { println!("connection error"); return () }
    }; 

    match ircstream.introduce(config.get_link_passwd(),
                              config.get_server_name(),
                              config.get_numeric(),
                              config.get_description()) {
        Ok(_) => (),
        Err(_) => { println!("introduce() failed"); return () }
    }

    enter_main_loop(config, ircstream);
}

fn load_config(file_path: &str) -> Result<Config> {
    Config::load(&Path::new(file_path))
}

fn enter_main_loop<T: ServerProtocol>(config: Config, mut ircstream: IrcStream<T>) {
    loop {
        match ircstream.recv_msg(config.get_encoding()) {
            Ok(unparsed_msg) => {
                if let Ok(irc_msg) = unparsed_msg {
                    match &irc_msg.command[..] {
                        "PRIVMSG" => println!("Got a private! How exciting!"),
                        _ => ()
                    }
                } else {
                    println!("Invalid IRC Message");
                    break;
                }
            }
            Err(e) => {
                println!("Connection reset by peer: {}", e.description());
                break;
            }
        }
    }
}
