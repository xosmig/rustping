use ::std::net::Ipv4Addr;
use ::libc::{self, AF_INET};
use ::std::mem;

pub trait IntoRaw {
    type Raw;
    fn into_raw(self) -> Self::Raw;
}

impl IntoRaw for Ipv4Addr {
    type Raw = libc::sockaddr_in;

    fn into_raw(self) -> Self::Raw {
        let mut res: Self::Raw = unsafe { mem::zeroed() };
        res.sin_family = AF_INET as _;
        res.sin_port = 0;
        res.sin_addr = libc::in_addr { s_addr: u32::from(self) };
        res
    }
}
