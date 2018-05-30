use std::net::Ipv4Addr;
use ::libc::{self, c_int, c_void, socklen_t};
use ::libc::{PF_INET, SOCK_RAW, SOCK_CLOEXEC, IPPROTO_ICMP};
use ::std::io;
use ::std::mem;
use ::sys_return::*;
use ::into_raw::IntoRaw;

#[derive(Copy, Clone)]
enum IcmpMessageType {
    EchoReply = 0,
    // Echo Reply
    DestinationUnreachable = 3,
    // Destination Unreachable
    Redirect = 5,
    // Redirect
    Echo = 8,
    // Echo
    RouterAdvertisement = 9,
    // Router Advertisement
    RouterSolicitation = 10,
    // Router Solicitation
    TimeExceeded = 11,
    // Time Exceeded
    ParameterProblem = 12,
    // Parameter Problem
    Timestamp = 13,
    // Timestamp
    TimestampReply = 14,
    // Timestamp Reply
    Photuris = 40,
    // Photuris
    ExtendedEchoRequest = 42,
    // Extended Echo Request
    ExtendedEchoReply = 43, // Extended Echo Reply
}

struct IcmpMessage {
    // ICMP type, see https://en.wikipedia.org/wiki/Internet_Control_Message_Protocol#Control_messages
    pub message_type: IcmpMessageType,
    // ICMP subtype, see https://en.wikipedia.org/wiki/Internet_Control_Message_Protocol#Control_messages
    pub code: u8,
    pub rest_of_header: [u8; 4],
    pub body: Box<[u8]>,
}

// The Internet Checksum is used, see https://tools.ietf.org/html/rfc1071
fn checksum(data: &[u8]) -> u16 {
    fn sum(a: u16, b: u16) -> u16 {
        let res = a as u32 + b as u32;
        let carry = res & (1 << 17);
        let res = res - carry + (carry >> 17);
        debug_assert!(res < (1 << 17));
        res as u16
    }

    let mut res: u16 = 0;
    for i in 0..(data.len() - 1) / 2 {
        res = sum(res, (data[2 * i + 1] << 8) as u16 | data[2 * i] as u16);
    }
    if data.len() % 2 == 1 {
        res = sum(res, data[data.len() - 1] as u16);
    }

    !res
}

impl IcmpMessage {
    pub fn marshal(&self) -> Box<[u8]> {
        let mut res = vec![self.message_type as u8, self.code as u8, 0, 0];
        res.extend_from_slice(&self.rest_of_header);
        res.extend_from_slice(self.body.as_ref());
        let cs = checksum(res.as_ref());
        res[2] = (cs & 0xFF) as u8;
        res[3] = (cs >> 8) as u8;
        res.into()
    }
}

struct IcmpSocket {
    sockfd: c_int,
}

impl IcmpSocket {
    fn new() -> io::Result<Self> {
        unsafe {
            let raw = sys_return_same(
                libc::socket(PF_INET, SOCK_RAW | SOCK_CLOEXEC, IPPROTO_ICMP))?;
            Ok(IcmpSocket { sockfd: raw })
        }
    }

    fn send_to(&mut self, msg: &IcmpMessage, addr: Ipv4Addr) -> io::Result<()> {
        let data = msg.marshal();
        let dest_addr = addr.into_raw();
        sys_return_unit(unsafe {
            ::libc::sendto(self.sockfd, data.as_ptr() as *const c_void, data.len(), /*flags=*/0,
                           &dest_addr as *const libc::sockaddr_in as *const libc::sockaddr,
                           mem::size_of_val(&dest_addr) as u32)
        })
    }

    fn setsockopt<T>(&mut self, level: c_int, optname: c_int, optval: T) -> io::Result<()> {
        sys_return_unit(unsafe {
            libc::setsockopt(self.sockfd, level, optname,
                             &optval as *const _ as *const c_void,
                             mem::size_of::<T>() as u32)
        })
    }

    pub fn getsockopt<T: Copy>(&mut self, level: c_int, optname: c_int) -> io::Result<T> {
        unsafe {
            let mut optval: T = mem::zeroed();
            let mut optlen = mem::size_of::<T>() as socklen_t;
            sys_return_unit(libc::getsockopt(self.sockfd, level, optname,
                                             &mut optval as *mut _ as *mut c_void,
                                             &mut optlen as *mut _))?;
            assert_eq!(optlen as usize, mem::size_of::<T>());
            Ok(optval)
        }
    }
}
