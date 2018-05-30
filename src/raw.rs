use ::std::net::Ipv4Addr;
use ::libc::{self, AF_INET};
use ::std::mem;
use ::std::io;

pub trait IntoRaw {
    type Raw;
    fn into_raw(self) -> Self::Raw;
}

pub trait FromRaw: ::std::marker::Sized {
    type Raw;
    fn from_raw(raw: Self::Raw) -> io::Result<Self>;
}

impl IntoRaw for Ipv4Addr {
    type Raw = libc::sockaddr_in;

    fn into_raw(self) -> Self::Raw {
        let mut res: Self::Raw = unsafe { mem::zeroed() };
        res.sin_family = AF_INET as _;
        res.sin_port = 0;
        res.sin_addr = libc::in_addr { s_addr: u32::from(self).to_be() };
        res
    }
}

impl FromRaw for Ipv4Addr {
    type Raw = libc::sockaddr_in;

    fn from_raw(raw: Self::Raw) -> io::Result<Self> {
        if raw.sin_family != AF_INET as _ {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Not a valid IPv4 address"));
        }
        Ok(Ipv4Addr::from(u32::from_be(raw.sin_addr.s_addr)))
    }
}
