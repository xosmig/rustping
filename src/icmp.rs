use ::std::net::Ipv4Addr;
use ::libc::{self, c_int, c_void, socklen_t};
use ::libc::{PF_INET, SOCK_RAW, SOCK_CLOEXEC, IPPROTO_ICMP};
use ::std::io;
use ::std::mem;
use ::sys_return::*;
use ::raw::{IntoRaw, FromRaw};
use ::num_traits::FromPrimitive;
use ::num_serialize::*;

pub use ::libc::{timeval, SOL_SOCKET, SO_SNDTIMEO, SO_RCVTIMEO};

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

pub struct Message {
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
        let carry = res & (1 << 16);
        let res = res - carry + (carry >> 16);
        debug_assert!(res < (1 << 16));
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

impl Message {
    pub fn marshal(&self) -> Box<[u8]> {
        let mut res = vec![self.message_type as u8, self.code];
        // place for the checksum
        res.extend_from_slice(&[0, 0]);
        res.extend_from_slice(&self.rest_of_header);
        res.extend_from_slice(self.body.as_ref());
        let cs = checksum(res.as_ref());
        res[2..4].copy_from_slice(&u16_to_be(cs));
        res.into()
    }

    pub fn parse(data: &[u8]) -> io::Result<Message> {
        if data.len() < 8 {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Message is too small"));
        }
        let cs = checksum(data);
        if cs != 0 && cs != !0 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid checksum"));
        }
        Ok(Message {
            message_type: MessageType::from_u8(data[0]).ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData,
                               format!("Invalid icmp message type: {}", data[0]))
            })?,
            code: data[1],
            rest_of_header: [data[4], data[5], data[6], data[7]],
            body: Box::from(&data[8..]),
        })
    }
}

pub struct Socket {
    sockfd: c_int,
}

fn cut_off_ipv4_header(data: &[u8]) -> io::Result<&[u8]> {
    if data.len() < 20 {
        return Err(io::Error::new(io::ErrorKind::InvalidData,
                                  "Expected IPv4 header at the beginning"));
    }
    Ok(&data[20..])
}

impl Socket {
    pub fn new() -> io::Result<Self> {
        unsafe {
            let raw = sys_return_same(
                libc::socket(PF_INET, SOCK_RAW | SOCK_CLOEXEC, IPPROTO_ICMP))?;
            Ok(Socket { sockfd: raw })
        }
    }

    pub fn send_to(&mut self, msg: &Message, addr: Ipv4Addr) -> io::Result<()> {
        let data = msg.marshal();
        let dest_addr = addr.into_raw();
        sys_return_unit(unsafe {
            libc::sendto(self.sockfd, data.as_ptr() as *const c_void, data.len(), /*flags=*/0,
                         &dest_addr as *const libc::sockaddr_in as *const libc::sockaddr,
                         mem::size_of_val(&dest_addr) as u32)
        })
    }

    pub fn recv_from(&mut self) -> io::Result<(Message, Ipv4Addr)> {
        let mut source_raw: libc::sockaddr_in = unsafe { mem::uninitialized() };
        let mut addrlen = mem::size_of_val(&source_raw) as socklen_t;
        let mut buf: [u8; 2048] = unsafe { mem::uninitialized() };

        let m_len = sys_return(unsafe {
            libc::recvfrom(self.sockfd, buf.as_mut_ptr() as *mut _, buf.len(), /*flags=*/ 0,
                           &mut source_raw as *mut libc::sockaddr_in as *mut libc::sockaddr,
                           &mut addrlen as *mut _)
        })?;
        let data = cut_off_ipv4_header(&buf[0..m_len])?;

        assert_eq!(addrlen as usize, mem::size_of_val(&source_raw));
        Ok((Message::parse(data)?, Ipv4Addr::from_raw(source_raw)?))
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

impl Drop for Socket {
    fn drop(&mut self) {
        unsafe {
            ::libc::close(self.sockfd);
        }
    }
}
