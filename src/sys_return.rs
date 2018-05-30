use ::std::io;
use ::num::{Integer, Zero, NumCast, ToPrimitive};

pub fn sys_return_same<T: Integer>(res: T) -> io::Result<T> {
    if res < Zero::zero() {
        Err(io::Error::last_os_error())
    } else {
        Ok(res)
    }
}

pub fn sys_return<T: Integer + ToPrimitive, R: NumCast>(res: T) -> io::Result<R> {
    return sys_return_same(res).map(|x| NumCast::from(x).unwrap())
}

pub fn sys_return_unit<T: Integer + ToPrimitive + NumCast>(res: T) -> io::Result<()> {
    sys_return_same(res).map(|_| ())
}
