extern crate libc;
extern crate nix;

use libc::{
    c_int,
    getpid,
    setsockopt,
};
use nix::{Error, Errno};
use nix::sys::socket::{
    AddressFamily, 
    SockAddr, 
    SockFlag, 
    SockType, 
};

use nix::sys::socket::{
    bind,
    recvmsg,
    sendmsg,
    socket,
};

#[repr(C)]
#[derive(Debug)]
pub enum NetlinkSockOpt {
    AddMembership = 1,
    DropMembership = 2,
    PktInfo = 3,
    BroadcastError = 4,
    NoEnobufs = 5,
}

fn setsockopt_int(
    fd: c_int,
    level: c_int, 
    option: c_int, 
    val: c_int
) -> Result<(), nix::Error> {
    use std::mem;
    let res = unsafe {
        setsockopt(
            fd, 
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

fn nl_open() -> Result<i32, nix::Error> {
    let sock = socket(
        AddressFamily::Netlink,
        SockType::Raw,
        SockFlag::empty(),
        libc::NETLINK_USERSOCK,
    )?; 

    let pid;
    unsafe {
        pid = getpid();
    };

    bind(
        sock, 
        &SockAddr::new_netlink(pid as u32, 0)
    )?;

    setsockopt_int(sock, 270, libc::NETLINK_ADD_MEMBERSHIP, 22)?;

    return Ok(sock);
}

fn nl_recv(sock: i32, count: u32) -> Result<(), nix::Error> {
    let mut buf = [0u8; 1024];
    let mut cmsg : nix::sys::socket::CmsgSpace<()> = nix::sys::socket::CmsgSpace::new();

    println!("[{}]", count);
    recvmsg(
        sock,
        &[nix::sys::uio::IoVec::from_mut_slice(&mut buf[..])],
        Some(&mut cmsg),
        nix::sys::socket::MsgFlags::empty(),
    )?;

    return match nl_send(sock, buf) {
        Ok(_) => Ok(()),
        Err(err) => Err(err),
    };
}

fn nl_send(sock: i32, buf: [u8; 1024]) -> Result<usize, nix::Error > {
    return sendmsg(
        sock,
        &[nix::sys::uio::IoVec::from_slice(&buf[..])],
        &[],
        nix::sys::socket::MsgFlags::empty(),
        None,
    );
}

fn main() {
    let sk;
    let sock_res = nl_open();
    match sock_res {
        Ok(sock) => {
            println!("sock {}", sock);
            sk = sock;
        },
        Err(Error::Sys(Errno::EPERM)) => {
            println!("Please run as root.");
            return;
        },
        Err(err) => {
            println!("error {}", err);
            return;
        },
    }

    let mut count = 0;
    loop {
        match nl_recv(sk, count) {
            Ok(_) => {count = count + 1}
            Err(e) => {
                println!("error {}", e);
                return;
            }
        };
    }
}
