#![allow(dead_code)]

#[macro_use]
extern crate enum_primitive_derive;
extern crate libc;
extern crate num;
extern crate num_traits;
#[macro_use]
extern crate clap;

mod icmp;
mod sys_return;
mod raw;
mod check;
mod pinger;
mod num_serialize;
mod ping;

use ::std::time::Duration;


fn main() {
    let matches = clap::App::new("rustping")
        .version("0.1")
        .about("send ICMP ECHO_REQUEST to network hosts.")
        .arg(clap::Arg::with_name("destination")
            .index(1)
            .required(true)
            .value_name("destination")
            .help("The host name or the ip address of the host to which echo requests are sent."))
        .arg(clap::Arg::with_name("count")
            .short("c")
            .long("count")
            .value_name("count")
            .default_value("0")
            .help("Stop after sending count ECHO_REQUEST packets."))
        .arg(clap::Arg::with_name("interval")
            .short("i")
            .long("interval")
            .value_name("interval")
            .default_value("1")
            .help("Wait interval seconds between sending each packet."))
        .arg(clap::Arg::with_name("timeout")
            .short("W")
            .long("timeout")
            .value_name("timeout")
            .default_value("3")
            .help("Time to wait for a response, in seconds."))
        .get_matches();

    let timeout = value_t_or_exit!(matches.value_of("timeout"), u64);
    let count = value_t_or_exit!(matches.value_of("count"), u64);

    let res = ping::ping(::std::io::stdout(), ping::Config {
        host: matches.value_of("destination").unwrap().to_string(),
        interval: Duration::from_secs(value_t_or_exit!(matches.value_of("interval"), u64)),
        timeout: if timeout == 0 { None } else { Some(Duration::from_secs(timeout)) },
        count: if count == 0 { None } else { Some(count) },
    });
    if let Err(e) = res {
        eprintln!("{}", e);
        ::std::process::exit(1);
    }
}
