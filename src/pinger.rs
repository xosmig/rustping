use ::std::net::Ipv4Addr;
use ::std::io;
use ::std::time::Duration;
use ::num_serialize::*;
use ::icmp;

pub struct Pinger {
    socket: icmp::Socket,
    seq_number: u32,
}

impl Pinger {
    pub fn new() -> io::Result<Pinger> {
        Ok(Pinger {
            socket: icmp::Socket::new()?,
            seq_number: 1,
        })
    }

    pub fn set_timeout(&mut self, timeout: Option<Duration>) -> io::Result<()> {
        let timeout = timeout.unwrap_or(Duration::from_secs(0));
        let tv = icmp::timeval {
            tv_sec: timeout.as_secs() as i64,
            tv_usec: 0,
        };
        self.socket.setsockopt(icmp::SOL_SOCKET, icmp::SO_SNDTIMEO, tv)?;
        self.socket.setsockopt(icmp::SOL_SOCKET, icmp::SO_RCVTIMEO, tv)?;
        Ok(())
    }

    pub fn ping_once(&mut self, dest_ip: Ipv4Addr) -> io::Result<()> {
        let seq_encoded = u32_to_be(self.seq_number);
        let msg = icmp::Message {
            message_type: icmp::MessageType::Echo,
            code: 0,
            rest_of_header: seq_encoded,
            body: Box::from([].as_ref()),
        };
        self.seq_number += 1;

        self.socket.send_to(&msg, dest_ip)?;
        loop {
            match self.socket.recv_from() {
                Ok((msg, _)) => {
                    if msg.message_type == icmp::MessageType::EchoReply
                        && msg.rest_of_header == seq_encoded {
                        break;
                    }
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    break;
                }
                Err(e) => {
                    eprintln!("Error receiving response: {}", e);
                }
            }
        }

        Ok(())
    }
}

