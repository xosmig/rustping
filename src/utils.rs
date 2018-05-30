use ::std::{io, fs, str};
use ::std::io::ErrorKind::InvalidData;

pub fn read_number<R: io::Read>(input: &mut R) -> io::Result<i64> {
    let mut buf = vec![0 as u8; 20];
    let read = input.read(&mut buf)?;

    let str: &str = str::from_utf8(&buf[0..read])
        .or_else(|e| Err(io::Error::new(InvalidData, e)))?
        .trim();
    str.parse()
        .or_else(|e| Err(io::Error::new(InvalidData, e)))
}
