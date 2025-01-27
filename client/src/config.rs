#![allow(unused)]

use clap::{value_t, App, Arg, ArgMatches};
use crate::processing::PostProcConfig;
use crate::vnc::Connection;

pub struct Config<'a> {
    pub connection: Connection<'a>,
    pub processing: PostProcConfig,

    pub rotate: i8,

    pub view_only: bool,
    pub touch_input: String,
}

impl Config<'static> {
    
    pub fn cli<'a>(matches: &'a ArgMatches) -> Config<'a> {
        let connection = Connection {
            host: matches.value_of("HOST").unwrap(),
            port: value_t!(matches.value_of("PORT"), u16).unwrap_or(5900),
            username: matches.value_of("USERNAME").clone(),
            password: matches.value_of("PASSWORD").clone(),
            exclusive: matches.is_present("EXCLUSIVE"),
        };
        let processing = PostProcConfig {
            contrast_exp: value_t!(matches.value_of("CONTRAST"), f32).unwrap_or(1.0),
            contrast_gray_point: value_t!(matches.value_of("GRAYPOINT"), f32).unwrap_or(224.0),
            white_cutoff: value_t!(matches.value_of("WHITECUTOFF"), u8).unwrap_or(255),
        };
        return Config{
            connection,
            processing,
            
            rotate: value_t!(matches.value_of("ROTATE"), i8).unwrap_or(1),
            view_only: matches.value_of("VIEW_ONLY")
            .unwrap_or("false").trim().parse().unwrap(),
            touch_input: matches.value_of("TOUCH_INPUT").unwrap_or("/dev/input/event1").to_string(),
        }
    }

    pub fn arguments() -> ArgMatches {
        App::new("einkvnc")
            .about("VNC client")
            .arg(
                Arg::with_name("HOST")
                    .help("server hostname or IP")
                    .required(true)
                    .index(1),
            )
            .arg(
                Arg::with_name("PORT")
                    .help("server port")
                    .long("port")
                    .default_value("5900")
                    .takes_value(true)
            )
            .arg(
                Arg::with_name("USERNAME")
                    .help("server username")
                    .long("username")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("PASSWORD")
                    .help("server password")
                    .long("password")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("EXCLUSIVE")
                    .help("request a non-shared session")
                    .long("exclusive"),
            )
            .arg(
                Arg::with_name("CONTRAST")
                    .help("apply a post processing contrast filter")
                    .long("contrast")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("GRAYPOINT")
                    .help("the gray point of the post processing contrast filter")
                    .long("graypoint")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("WHITECUTOFF")
                    .help("apply a post processing filter to turn colors greater than the specified value to white (255)")
                    .long("whitecutoff")
                    .takes_value(true),
            ).arg(
                Arg::with_name("ROTATE")
                    .help("rotation (1-4), tested on a Clara HD, try at own risk")
                    .long("rotate")
                    .takes_value(true),
            ).arg(
                Arg::with_name("VIEW_ONLY")
                    .help("use VNC only as viewer, never sending any inputs?")
                    .default_value("false")
                    .long("viewonly")
                    .takes_value(true),
            ).arg(
                Arg::with_name("TOUCH_INPUT")
                    .help("the device that provides touch inputs.")
                    .default_value("/dev/input/event1")
                    .long("touch")
                    .takes_value(true),
            ).arg( // fake arg; making `cross run -- localhost` possible despite our always present arm release target.
                Arg::with_name("target")
                    .long("target")
                    .takes_value(true)
                    .hide(true)
            )
            .get_matches()
    }

}
