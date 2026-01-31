use anyhow::{Result, bail};
use libc::{AF_INET, EPOLL_CLOEXEC, EPOLL_CTL_ADD, EPOLL_CTL_MOD, EPOLLERR, EPOLLHUP, EPOLLIN, EPOLLONESHOT, EPOLLOUT, EPOLLWRNORM, INADDR_ANY, INADDR_LOOPBACK, SOCK_NONBLOCK, SOCK_STREAM, accept4, bind, epoll_create1, epoll_ctl, epoll_event, epoll_wait, htonl, htons, in_addr, in_addr_t, listen, sockaddr, sockaddr_in, socket};

use std::{io::Error, thread, time::Duration, u16};

mod event;
fn main() -> Result<()> {
    let epoll_fd = unsafe{epoll_create1(0)};
    if epoll_fd < 0 {
        bail!("could not instantiate epoll: {}", Error::last_os_error());
    };

    let socket_fd = unsafe {socket(AF_INET, SOCK_STREAM, 0)};
    if socket_fd < 0 {
        bail!("could not instantiate a socket: {}", Error::last_os_error());
    };

    let address_to_listen = sockaddr_in {
        sin_family: AF_INET as u16,
        sin_port: htons(8080), //endianness-safe
        sin_addr: in_addr{s_addr: INADDR_ANY},
        sin_zero: [0u8; 8]
    };

    
    //listen!
    let bind_result =unsafe {bind(socket_fd, &address_to_listen as *const sockaddr_in  as *const sockaddr, size_of::<sockaddr_in>() as u32)};
    if bind_result < 0 {
        bail!("could not bind the TCP to given address: {}", Error::last_os_error());
    }

    let listen_result = unsafe {listen(socket_fd, 10)};
    if listen_result < 0 {
        bail!("could not listen the TCP socket on given address: {}", Error::last_os_error());
    }

    let mut event = epoll_event{
        events: (EPOLLIN | EPOLLOUT | EPOLLONESHOT) as u32,
        u64: socket_fd as u64
    };
     let add_ctl = unsafe {epoll_ctl(epoll_fd, EPOLL_CTL_ADD, socket_fd, &mut event)};
     if add_ctl < 0 {
        bail!("could not register interest in incoming tcp requests: {}", Error::last_os_error());
     }
     let mut epoll_event_recieved = epoll_event {
        events: 0,
        u64: socket_fd  as u64
     };
     let mut open_connections: Vec<i32> = Vec::new();
    loop {
        println!("listening");
      let events_count =  unsafe {epoll_wait(epoll_fd, &mut epoll_event_recieved, 10, -1);};
      println!("{:?}", epoll_event_recieved);
      let connectionfd = unsafe { accept4(socket_fd, std::ptr::null_mut(), std::ptr::null_mut(), SOCK_NONBLOCK) };
      if connectionfd < 0 {
        println!("could not accept connection");
        continue;
      }
      open_connections.push(connectionfd);
      println!("{}", connectionfd);
      unsafe {epoll_ctl(epoll_fd, EPOLL_CTL_MOD, socket_fd, &mut epoll_event_recieved)};
      println!("recieved!")
    }
   
}
