extern crate libc;
use libc::c_int;
extern crate nix;
use nix::sys::socket;

type Result<T> = std::result::Result<T, nix::Error>;

#[derive(Debug)]
pub struct Socket(c_int);

impl Socket {
    fn __new() -> Result<Self> {
        let fd = socket::socket(
            nix::sys::socket::AddressFamily::Netlink,
            nix::sys::socket::SockType::Raw,
            nix::sys::socket::SockFlag::empty(),
            libc::NETLINK_USERSOCK,
        )?;

        let pid = unsafe { libc::getpid() };

        socket::bind(fd, &nix::sys::socket::SockAddr::new_netlink(pid as u32, 0))?;

        Ok(Socket(fd))
    }

    pub fn new() -> Result<Self> {
        let s = Self::__new()?;
        s.setsockopt_int(270, libc::NETLINK_ADD_MEMBERSHIP, 22)?;
        Ok(s)
    }

    fn setsockopt_int(&self, level: c_int, option: c_int, val: c_int) -> Result<()> {
        use std::mem;
        let res = unsafe {
            libc::setsockopt(
                self.0,
                level,
                option as c_int,
                mem::transmute(&val),
                mem::size_of::<c_int>() as u32,
            )
        };

        if res == -1 {
            return Err(nix::Error::last());
        }

        Ok(())
    }

    fn recv(&self, buf: &mut [u8]) -> Result<usize> {
        socket::recvmsg::<()>(
            self.0,
            &[nix::sys::uio::IoVec::from_mut_slice(&mut buf[..])],
            None,
            nix::sys::socket::MsgFlags::empty(),
        ).map(|r| r.bytes)
    }

    fn send(&self, _: Option<u16>, buf: &[u8]) -> Result<()> {
        socket::sendmsg(
            self.0,
            &[nix::sys::uio::IoVec::from_slice(&buf[..])],
            &[],
            nix::sys::socket::MsgFlags::empty(),
            None,
        ).map(|_| ())
    }

    //fn close(&self) -> Result<()> {
    //    return socket::shutdown(self.0, nix::sys::socket::Shutdown::Both);
    //}
}

fn main() {
    use std::sync::Arc;
    let sk = Arc::new(Socket::new().unwrap());
    let sk1 = sk.clone();

    use std::sync::mpsc;
    let (tx, rx): (mpsc::Sender<Vec<u8>>, mpsc::Receiver<Vec<u8>>) = mpsc::channel();
    use std::thread;
    thread::spawn(move || {
        let mut count = 0;
        let mut buf = vec![0u8; 1024];
        loop {
            print!("[{}] ", count);
            let len = sk.recv(&mut buf).unwrap();
            buf.truncate(len);
            count += 1;
            println!("{} B: {:?}", len, &buf[0x10..]);
            tx.send(buf.clone()).unwrap();
        }
    });

    for m in rx.iter() {
        sk1.send(None, &m[..]).unwrap();
    }
}
