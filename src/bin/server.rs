use anyhow::{Result, bail};
use libc::{
    AF_INET, EPOLL_CLOEXEC, EPOLL_CTL_ADD, EPOLL_CTL_DEL, EPOLL_CTL_MOD, EPOLLERR, EPOLLHUP, EPOLLIN, EPOLLONESHOT, EPOLLOUT, EPOLLWRNORM, INADDR_ANY, INADDR_LOOPBACK, SOCK_NONBLOCK, SOCK_STREAM, accept4, bind, close, epoll_create1, epoll_ctl, epoll_event, epoll_wait, htonl, htons, in_addr, in_addr_t, listen, read, sockaddr, sockaddr_in, socket
};

use std::{error, io::{Error, ErrorKind}, ops::Index, os::raw::c_void, ptr::null_mut, thread, time::Duration, u16};
fn main() -> Result<()> {
    let epoll_fd = unsafe { epoll_create1(0) };
    if epoll_fd < 0 {
        bail!("could not instantiate epoll: {}", Error::last_os_error());
    };

    let socket_fd = unsafe { socket(AF_INET, SOCK_STREAM | SOCK_NONBLOCK, 0) };
    if socket_fd < 0 {
        bail!("could not instantiate a socket: {}", Error::last_os_error());
    };

    let address_to_listen = sockaddr_in {
        sin_family: AF_INET as u16,
        sin_port: htons(8080), //endianness-safe
        sin_addr: in_addr { s_addr: INADDR_ANY },
        sin_zero: [0u8; 8],
    };

    //listen!
    let bind_result = unsafe {
        bind(
            socket_fd,
            &address_to_listen as *const sockaddr_in as *const sockaddr,
            size_of::<sockaddr_in>() as u32,
        )
    };
    if bind_result < 0 {
        bail!(
            "could not bind the TCP to given address: {}",
            Error::last_os_error()
        );
    }

    let listen_result = unsafe { listen(socket_fd, 10) };
    if listen_result < 0 {
        bail!(
            "could not listen the TCP socket on given address: {}",
            Error::last_os_error()
        );
    }

    let mut event = epoll_event {
        events: (EPOLLIN ) as u32,
        u64: socket_fd as u64,
    };
    let add_ctl = unsafe { epoll_ctl(epoll_fd, EPOLL_CTL_ADD, socket_fd, &mut event) };
    if add_ctl < 0 {
        bail!(
            "could not register interest in incoming tcp requests: {}",
            Error::last_os_error()
        );
    }
    let mut epoll_event_recieved = [epoll_event {
        events: 0,
        u64: socket_fd as u64,
    }; 10];

    let mut open_connections: Vec<i32> = Vec::new();


    loop {
        println!("listening");
        let events_count = unsafe {
            epoll_wait(epoll_fd, epoll_event_recieved.as_mut_ptr(), 10, -1)
        };
        println!("woke up for a total of {events_count} events");
        for i in 0..events_count as usize {
            let flags = epoll_event_recieved[i].events;
            let concerned_fd = epoll_event_recieved[i].u64;
            println!("{:?}", epoll_event_recieved[i]);
            if flags & EPOLLIN as u32 != 0{
                if concerned_fd != socket_fd as u64{
                    //this event concerns a connection: either FIN or regular data
                    let mut buf: [u8; 256] = [0;256];
                    //make sure to drain completely
                    let mut first_read_response: Option<isize> = None;
                    loop {
                        //VERY IMP: READ ERROR DOES NOT NECESSERILIY MEAN ERROR, IT JUST MEANS THERE IS NO MORE DATA **YET**
                        let read_response = unsafe {
                            read(concerned_fd as i32, buf.as_mut_ptr() as *mut _, buf.len())
                        };
                        let is_noncarryable_error = read_response == -1 && Error::last_os_error().kind() != ErrorKind::WouldBlock;
                        if first_read_response.is_none() || is_noncarryable_error {
                            //why the second condition ? because if its a genuine error, no point in continuing
                            first_read_response = Some(read_response)
                        }
                        if read_response <= 0 {
                            //EOF or ERROR, no need to continue either way
                            break
                        }
                    }
                    if first_read_response.unwrap() == 0 {
                        //EOF, CLIENT SENT A FIND
                        println!("the connection {concerned_fd} sent a FIN. Removing and closing it");
                        let con_index = open_connections.iter().position(|&e| e == concerned_fd as i32).unwrap();
                        open_connections.remove(con_index);
                        unsafe {epoll_ctl(epoll_fd, EPOLL_CTL_DEL, concerned_fd as i32, &mut ConnectionEvent(concerned_fd))};
                        unsafe {close(concerned_fd as i32)};
                    }else if first_read_response.unwrap() == -1{
                        //most likely: it's a blockin call prevented, try again later. might be a genuine error in some cases
                        println!("{}", Error::last_os_error());
                        println!("error reading connection {concerned_fd}");
                    }else {
                        //everythin was fine
                        println!("connection {concerned_fd} sent some data: {:?}", String::from_utf8(buf.to_vec()))
                    }
                }else {
                      //connection request
                let connection_fd = unsafe {accept4(socket_fd, null_mut(), null_mut(), SOCK_NONBLOCK)};
                if (connection_fd > 0) && (unsafe {epoll_ctl(epoll_fd, EPOLL_CTL_ADD, connection_fd, &mut ConnectionEvent(connection_fd as u64))}) > -1{
                    println!("new connection accepted: {}", connection_fd);
                    open_connections.push(connection_fd);
                }else{
                    println!("something went wrong registering a new connection")
                };
                }
              
                
            }else{
                println!("recieved something else")
            }
        };
        println!("recieved!")
    }
}

fn ConnectionEvent(con_fd: u64) -> epoll_event {
    epoll_event { events: (EPOLLIN as u32), u64: con_fd }
}