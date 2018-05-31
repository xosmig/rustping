#![allow(dead_code)]

#[macro_use]
extern crate enum_primitive_derive;
extern crate libc;
extern crate num;
extern crate num_traits;
extern crate clap;

mod icmp;
mod sys_return;
mod raw;
mod check;
mod pinger;
mod num_serialize;

use ::std::net::Ipv4Addr;
use ::check::Check;
use ::pinger::Pinger;
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
            .help("Stop after sending count ECHO_REQUEST packets."))
        .arg(clap::Arg::with_name("interval")
            .short("i")
            .long("interval")
            .value_name("interval")
            .help("Wait interval seconds between sending each packet."))
        .arg(clap::Arg::with_name("timeout")
            .short("W")
            .long("timeout")
            .value_name("timeout")
            .help("Time to wait for a response, in seconds."))
        .get_matches();

    let dest_ip: Ipv4Addr = matches.value_of("destination").unwrap().parse()
        .check("Can't parse ip address");

    let mut pinger = Pinger::new().expect("FOO");
    pinger.set_timeout(Some(Duration::from_secs(1))).expect("BAZ");
    pinger.ping_once(dest_ip).expect("BAR");
}
