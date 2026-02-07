use std::{io::Error, os::raw::c_void, thread, time::Duration};

use anyhow::Result;
use libc::{
    AF_INET, EINPROGRESS, EPOLL_CTL_MOD, EPOLLIN, EPOLLOUT, SO_ERROR, SOCK_NONBLOCK, SOCK_STREAM,
    SOL_SOCKET, close, connect, epoll_create1, epoll_ctl, epoll_event, epoll_wait, getsockopt,
    htons, in_addr, sockaddr, sockaddr_in, socket, socklen_t,
};
use std::net::Ipv4Addr;
#[path = "../epoll.rs"]
mod epoll;
use epoll::{new_epoll_event, register_interest};

use crate::epoll::has_flag;
fn main() -> Result<()> {
    let n_connections: u32 = 1000;
    let epollfd = unsafe { epoll_create1(0) };
    let mut connections_active: Vec<i32> = Vec::new();

    //1. initialize all the connections to the server, and store thier fds
    //todo: start connections in parallel, using threads
    for i in 1..=n_connections {
        let connection_socket_fd = unsafe { socket(AF_INET, SOCK_STREAM | SOCK_NONBLOCK, 0) };
        if connection_socket_fd == -1 {
            println!(
                "could not initiate a socket: number: {}, error: {}",
                i,
                Error::last_os_error()
            );
        }
        //our socket -> server
        let addr = sockaddr_in {
            sin_family: AF_INET as u16,
            sin_port: htons(8080), //endianness-safe
            sin_addr: in_addr {
                s_addr: Ipv4Addr::new(0, 0, 0, 0).to_bits(),
            },
            sin_zero: [0; 8],
        };
        let connection_response = unsafe {
            connect(
                connection_socket_fd,
                &addr as *const sockaddr_in as *const sockaddr,
                size_of::<sockaddr_in>() as u32,
            )
        };
        if connection_response == -1 {
            if Error::last_os_error().raw_os_error().unwrap() == EINPROGRESS {
                //error kind not yet available in stable
                register_interest(
                    epollfd,
                    connection_socket_fd,
                    &mut new_epoll_event(EPOLLOUT | EPOLLIN, connection_socket_fd),
                )?;
                println!(
                    "the connection {} is in progress, registered interest.",
                    connection_socket_fd
                );
            } else {
                println!(
                    "could not connect the socket {} to server: {}",
                    connection_socket_fd,
                    Error::last_os_error()
                );
            }
        } else {
            //0, connection succesfull
            println!("client {} connected to server.", i);
            connections_active.push(connection_socket_fd);
        }
    }

    //2. the main loop
    loop {
        //todo: a threadpool for further time-cutting
        let mut events_buffer: [epoll_event; 5000] = [new_epoll_event(0, 0); 5000]; //rest will round robin, no starvation :)
        let events_occured =
            unsafe { epoll_wait(epollfd, events_buffer.as_mut_ptr(), 5000, -1) } as usize;
        println!("woke up for {events_occured} events");
        for i in 0..events_occured {
            //we know that the event is bound to take place on a connection fd, no verif required
            let flags = events_buffer[i].events;
            let concerned_fd = events_buffer[i].u64 as i32;
            if has_flag(flags, EPOLLOUT) {
                //something happened with the connection we previously requested
                let mut err: i32 = 0; // usually i32 for SO_ERROR
                let mut len: socklen_t = std::mem::size_of_val(&err) as socklen_t;
                if unsafe {
                    getsockopt(
                        concerned_fd,
                        SOL_SOCKET,
                        SO_ERROR,
                        &mut err as *mut i32 as *mut c_void,
                        &mut len as *mut socklen_t,
                    )
                } == -1
                {
                    println!(
                        "could not check socketopt: {}. error: {}",
                        concerned_fd,
                        Error::last_os_error()
                    )
                }
                if err == 0 {
                    //socket started soccuesfully!
                    //remove EPOLLOUT to stop continuous wakeups
                    unsafe {
                        epoll_ctl(
                            epollfd,
                            EPOLL_CTL_MOD,
                            concerned_fd,
                            &mut new_epoll_event(EPOLLIN, concerned_fd),
                        )
                    };
                    connections_active.push(concerned_fd);
                    println!("connection succesfully made: {}", concerned_fd)
                } else {
                    println!("something went wrong with connection: {}", concerned_fd);
                    unsafe { close(concerned_fd) };
                }
            }
        }
    }
    //CONFIRMED: does not send FIN until process exits
    Ok(())
}
