#![feature(io)]
#![feature(core)]

extern crate "rustc-serialize" as rustc_serialize;
extern crate encoding;
#[cfg(feature = "ssl")]
extern crate openssl;
extern crate time;

mod irc;
mod cmd;
mod conf;
mod protocol;

use irc::IrcStream;
use conf::Config;
use std::io::Result;
use std::path::Path;
use std::error::Error;
use std::rc::Rc;
use std::cell::RefCell;

use protocol::unreal::Unreal;
use protocol::ServerProtocol;

// TODO deal with case-sensitiveness?
// TODO Disconnect / netsplit / reconnect and resync

fn main() {

    // TODO make cmd-line option to pass conf file path
    // TODO Better and more descriptive error handling

    let config = Rc::new(RefCell::new(match load_config("tools.conf") {
        Ok(cfg) => cfg,
        // An ugly hack to get around the following warning:
        // --
        // use of deprecated item: use the Error trait's description method instead, #[warn(deprecated)] on by default
        //--
        // std::io::Error will go away at some point; when it does, make it e.description() again
        Err(e) => { println!("ERROR loading conf: {}", (&e as &Error).description()); return () }
    }));

    // TODO Do not hardcode protocol handler
    let ircstream = match IrcStream::new(config.clone(), Unreal::new(config.clone())) {
        Ok(stream) => stream,
        Err(_) => { println!("connection error"); return () }
    }; 

    match ircstream.introduce() {
        Ok(_) => (),
        Err(_) => { println!("introduce() failed"); return () }
    }

    enter_main_loop(ircstream);
}

fn load_config(file_path: &str) -> Result<Config> {
    Config::load(&Path::new(file_path))
}

fn enter_main_loop<T: ServerProtocol>(ircstream: IrcStream<T>) {
    for message in ircstream.iter() {
        match message {
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
                println!("Connection reset by peer: {}", (&e as &Error).description());
                break;
            }
        }        
    }
}
