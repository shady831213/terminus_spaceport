extern crate tun_tap;
extern crate libc;

use tun_tap::{Iface, Mode};
use self::libc::c_int;
use std::ops::Deref;
use std::os::unix::io::AsRawFd;

pub struct TunTap {
    iface: Iface
}

impl TunTap {
    pub fn new(ifname: &str, mode: Mode, packet_info: bool, nonblock: bool) -> std::io::Result<TunTap> {
        let iface = if packet_info {
            Iface::new(ifname, mode)?
        } else {
            Iface::without_packet_info(ifname, mode)?
        };
        if nonblock {
            let fd = iface.as_raw_fd();
            let mut nonblock: c_int = 1;
            let result = unsafe { libc::ioctl(fd, libc::FIONBIO, &mut nonblock) };
            if result == -1 {
                return Err(std::io::Error::last_os_error());
            }
        }
        Ok(TunTap { iface })
    }
}

impl Deref for TunTap {
    type Target = Iface;
    fn deref(&self) -> &Self::Target {
        &self.iface
    }
}