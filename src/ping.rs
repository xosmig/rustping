use ::std::net::{Ipv4Addr, IpAddr, ToSocketAddrs};
use ::std::time::Duration;
use ::pinger::Pinger;
use ::std::{io, time, thread};


#[derive(Clone, Debug, Default)]
pub struct Config {
    pub host: String,
    pub interval: Duration,
    pub timeout: Option<Duration>,
    pub count: Option<u64>
}

fn get_ipv4_by_host(host: &str) -> io::Result<Ipv4Addr> {
    let mut dest_addr_iter = (host, 0).to_socket_addrs()?.filter_map(|sock_addr| {
        if let IpAddr::V4(addr_v4) = sock_addr.ip() {
            return Some(addr_v4);
        }
        None
    });
    dest_addr_iter.next().ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, format!("Host '{}' not found", host))
    })
}

pub fn ping<W: io::Write>(mut output: W, config: Config) -> io::Result<()> {
    let dest_ip = get_ipv4_by_host(&config.host)?;

    let mut pinger = Pinger::new()?;
    if let Some(timeout) = config.timeout {
        pinger.set_timeout(Some(timeout))?;
    }

    writeln!(output, "PING {} ({})", config.host, dest_ip)?;

    let n_attempts = if let Some(n) = config.count { n } else { u64::max_value() };
    for i in 0..n_attempts {
        println!("Request {}:", i + 1);
        let start = time::Instant::now();
        match pinger.ping_once(dest_ip) {
            Ok(()) => writeln!(output, "Ok")?,
            Err(e) => writeln!(output, "Error: {}", e)?,
        };

        let length = start.elapsed();
        if config.interval > length {
            thread::sleep(config.interval - length);
        }
    }

    Ok(())
}
