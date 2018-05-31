#![allow(dead_code)]
#![feature(ip_constructors)]

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

use ::std::net::Ipv4Addr;
use ::check::Check;


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
            .default_value("-1")
            .help("Stop after sending count ECHO_REQUEST packets."))
        .arg(clap::Arg::with_name("interval")
            .short("i")
            .long("interval")
            .value_name("interval")
            .default_value("1")
            .help("Wait interval seconds between sending each packet."))
        .arg(clap::Arg::with_name("deadline")
            .short("w")
            .long("deadline")
            .value_name("deadline")
            .default_value("-1")
            .help("Specify a timeout, in seconds, before the program exits regardless of how \
                many packets have been sent or received."))
        .arg(clap::Arg::with_name("timeout")
            .short("W")
            .long("timeout")
            .value_name("timeout")
            .default_value("-1")
            .help("Time to wait for a response, in seconds."))
        .get_matches();

    let dest_ip: Ipv4Addr = matches.value_of("destination").unwrap().parse()
        .check("Can't parse ip address");

    let msg = icmp::IcmpMessage {
        message_type: icmp::MessageType::Echo,
        code: 0,
        rest_of_header: [0, 0, 0, 0],
        body: Box::from([].as_ref()),
    };
    let mut socket = icmp::IcmpSocket::new().expect("Error opening icmp socket");
    socket.send_to(&msg, dest_ip).expect("Error sending echo request");
}
