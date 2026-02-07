use std::{io::{Error, Write}, net::TcpStream, thread, time::Duration};

use anyhow::{Result};
use std::net::{Ipv4Addr};
use libc::{AF_INET, SOCK_NONBLOCK, SOCK_STREAM, connect, epoll_create1, htons, in_addr, sockaddr, sockaddr_in, socket};

fn main() -> Result<()>{
    let nConnections: u32 = 50000;
    let epollfd = unsafe {
     epoll_create1(0)   
    };
    let mut connections_active: Vec<i32> = Vec::new();

    //1. initialize all the connections to the server, and store thier fds
    for i in 1..=nConnections {
        let connection_socket_fd = unsafe {socket(AF_INET, SOCK_STREAM, 0)};
        if connection_socket_fd == -1 {
            println!("could not initiate a socket: number: {}, error: {}", i, Error::last_os_error());
        }
        //our socket -> server
        let addr = sockaddr_in {
                sin_family: AF_INET as u16,
                sin_port: htons(8080), //endianness-safe
                sin_addr: in_addr { s_addr: Ipv4Addr::new(0, 0, 0, 0).to_bits() },
                sin_zero: [0; 8]
            };
        if (unsafe{connect(connection_socket_fd,  &addr as *const sockaddr_in as *const sockaddr, size_of::<sockaddr_in>() as u32)} != -1) {
            println!("client {} connected to server.", i);
            connections_active.push(connection_socket_fd);
        }else{
            println!("error connecting socket {} to server: {}", i, Error::last_os_error());
        }

    }


    //CONFIRMED: does not send FIN until process exits
    thread::sleep(Duration::from_secs(100));
    Ok(())
}