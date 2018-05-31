use std::net::Ipv4Addr;
use ::libc::{self, c_int, c_void, socklen_t};
use ::libc::{PF_INET, SOCK_RAW, SOCK_CLOEXEC, IPPROTO_ICMP};
use ::std::io;
use ::std::mem;
use ::sys_return::*;
use ::raw::{IntoRaw, FromRaw};
use num_traits::FromPrimitive;


#[derive(Copy, Clone, Debug, Eq, PartialEq, Primitive)]
pub enum MessageType {
    // Echo Reply
    EchoReply = 0,
    // Destination Unreachable
    DestinationUnreachable = 3,
    // Redirect
    Redirect = 5,
    // Echo
    Echo = 8,
    // Router Advertisement
    RouterAdvertisement = 9,
    // Router Solicitation
    RouterSolicitation = 10,
    // Time Exceeded
    TimeExceeded = 11,
    // Parameter Problem
    ParameterProblem = 12,
    // Timestamp
    Timestamp = 13,
    // Timestamp Reply
    TimestampReply = 14,
    // Photuris
    Photuris = 40,
    // Extended Echo Request
    ExtendedEchoRequest = 42,
    // Extended Echo Reply
    ExtendedEchoReply = 43,
}

fn be_to_u16(be: &[u8]) -> u16 {
    (be[0] as u16) + ((be[1] as u16) << 8)
}

fn u16_to_be(val: u16, be: &mut [u8]) {
    be[0] = (val & 0xFF) as u8;
    be[1] = (val >> 8) as u8;
}

pub struct IcmpMessage {
    // ICMP type, see https://en.wikipedia.org/wiki/Internet_Control_Message_Protocol#Control_messages
    pub message_type: MessageType,
    // ICMP subtype, see https://en.wikipedia.org/wiki/Internet_Control_Message_Protocol#Control_messages
    pub code: u8,
    pub rest_of_header: [u8; 4],
    pub body: Box<[u8]>,
}

// The Internet Checksum is used, see https://tools.ietf.org/html/rfc1071
fn checksum(data: &[u8]) -> u16 {
    fn sum(a: u16, b: u16) -> u16 {
        let res = (a as u32) + (b as u32);
        let carry = res & (1 << 17);
        let res = res - carry + (carry >> 17);
        debug_assert!(res < (1 << 17));
        res as u16
    }

    let mut res: u16 = 0;
    for i in 0..(data.len() - 1) / 2 {
        res = sum(res, be_to_u16(&data[2 * i..]));
    }
    if data.len() % 2 == 1 {
        // values are in big-endian, hence the lonely byte
        // represents the least-significant byte of a 16-bit-wise word.
        res = sum(res, data[data.len() - 1] as u16);
    }

    !res
}

impl IcmpMessage {
    pub fn marshal(&self) -> Box<[u8]> {
        let mut res = vec![self.message_type as u8, self.code];
        // place for the checksum
        res.extend_from_slice(&[0, 0]);
        res.extend_from_slice(&self.rest_of_header);
        res.extend_from_slice(self.body.as_ref());

        u16_to_be(checksum(res.as_ref()), &mut res[2..4]);
        res.into()
    }

    pub fn parse(data: &[u8]) -> io::Result<IcmpMessage> {
        if data.len() < 8 {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Message is too small"));
        }
        let cs = be_to_u16(&data[2..4]);
        if checksum(data) != cs {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid checksum"));
        }
        Ok(IcmpMessage {
            message_type: MessageType::from_u8(data[0]).ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "Invalid icmp message type")
            })?,
            code: data[1],
            rest_of_header: [data[4], data[5], data[6], data[7]],
            body: Box::from(&data[8..]),
        })
    }
}

pub struct IcmpSocket {
    sockfd: c_int,
}

impl IcmpSocket {
    pub fn new() -> io::Result<Self> {
        unsafe {
            let raw = sys_return_same(
                libc::socket(PF_INET, SOCK_RAW | SOCK_CLOEXEC, IPPROTO_ICMP))?;
            Ok(IcmpSocket { sockfd: raw })
        }
    }

    pub fn send_to(&mut self, msg: &IcmpMessage, addr: Ipv4Addr) -> io::Result<()> {
        let data = msg.marshal();
        let dest_addr = addr.into_raw();
        sys_return_unit(unsafe {
            libc::sendto(self.sockfd, data.as_ptr() as *const c_void, data.len(), /*flags=*/0,
                         &dest_addr as *const libc::sockaddr_in as *const libc::sockaddr,
                         mem::size_of_val(&dest_addr) as u32)
        })
    }

    pub fn recv_from(&mut self) -> io::Result<(IcmpMessage, Ipv4Addr)> {
        let mut source_raw: libc::sockaddr_in = unsafe { mem::uninitialized() };
        let mut addrlen = mem::size_of_val(&source_raw) as socklen_t;
        let mut buf: [u8; 1024] = unsafe { mem::uninitialized() };

        sys_return_unit(unsafe {
            libc::recvfrom(self.sockfd, buf.as_mut_ptr() as *mut _, buf.len(), /*flags=*/ 0,
                           &mut source_raw as *mut libc::sockaddr_in as *mut libc::sockaddr,
                           &mut addrlen as *mut _)
        })?;

        assert_eq!(addrlen as usize, mem::size_of_val(&source_raw));
        Ok((IcmpMessage::parse(&buf)?, Ipv4Addr::from_raw(source_raw)?))
    }

    pub fn setsockopt<T>(&mut self, level: c_int, optname: c_int, optval: T) -> io::Result<()> {
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
