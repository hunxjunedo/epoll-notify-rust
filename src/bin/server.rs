use anyhow::{Ok, Result, bail};
use libc::{
    AF_INET, EPOLLIN, INADDR_ANY, SOCK_NONBLOCK, SOCK_STREAM,
    accept4, bind,  close, epoll_create1, epoll_ctl, epoll_event, epoll_wait, htons, in_addr,
    listen, read, sockaddr, sockaddr_in, socket,
};
#[path = "../epoll.rs"]
mod epoll;
use epoll::{register_interest, unregister_interest, new_epoll_event};
use std::{
    collections::HashMap, io::{Error, ErrorKind}, ptr::null_mut, u16
};

use crate::epoll::has_flag;

fn main() -> Result<()> {
    //epoll instantiate
    let epoll_fd = unsafe { epoll_create1(0) };
    if epoll_fd < 0 {
        bail!("could not instantiate epoll: {}", Error::last_os_error());
    };
    let socket_fd = start_listening(8080)?;

    //this encapsulates our interests
    let mut event = new_epoll_event(EPOLLIN, socket_fd);
    register_interest(epoll_fd, socket_fd, &mut event)?;

    let mut epoll_event_recieved = [new_epoll_event(0, 0); 50000];
    let mut open_connections: HashMap<i32, String> = HashMap::new();
    let mut unauthenticated_connections: Vec<i32> = Vec::new();

    loop {
        println!("listening");
        let events_count =
            unsafe { epoll_wait(epoll_fd, epoll_event_recieved.as_mut_ptr(), 50000, -1) };
        println!("woke up for a total of {events_count} events");
        for i in 0..events_count as usize {
            let flags = epoll_event_recieved[i].events;
            let concerned_fd = epoll_event_recieved[i].u64;
            // println!("{:?}", epoll_event_recieved[i]);
            event_handler(
                flags,
                concerned_fd as i32,
                socket_fd,
                epoll_fd,
                &mut open_connections,
                &mut unauthenticated_connections
            )?;
        }
        println!("recieved!")
    }
}

fn start_listening(port: u16) -> Result<i32> {
    let socket_fd = unsafe { socket(AF_INET, SOCK_STREAM | SOCK_NONBLOCK, 0) };
    if socket_fd < 0 {
        bail!("could not instantiate a socket: {}", Error::last_os_error());
    };

    let address_to_listen = sockaddr_in {
        sin_family: AF_INET as u16,
        sin_port: htons(port), //endianness-safe
        sin_addr: in_addr { s_addr: INADDR_ANY },
        sin_zero: [0u8; 8],
    };

    //listen!
    if unsafe {
        bind(
            socket_fd,
            &address_to_listen as *const sockaddr_in as *const sockaddr,
            size_of::<sockaddr_in>() as u32,
        )
    } < 0
    {
        bail!(
            "could not bind the TCP to given address: {}",
            Error::last_os_error()
        );
    }

    if unsafe { listen(socket_fd, 10) } < 0 {
        bail!(
            "could not listen the TCP socket on given address: {}",
            Error::last_os_error()
        );
    };
    Ok(socket_fd)
}


fn event_handler(
    flags: u32,
    concerned_fd: i32,
    socket_fd: i32,
    epoll_fd: i32,
    open_connections: &mut HashMap<i32, String>,
    unauthenticated_connections: &mut Vec<i32>
) -> Result<()> {
    if has_flag(flags, EPOLLIN) {
        if concerned_fd != socket_fd {
            let recieved_data = handle_data_on_connection(concerned_fd, open_connections, epoll_fd);
            if recieved_data.is_ok(){
                println!("{}",recieved_data.unwrap());
            };
        } else {
            //connection request
            handle_new_connection_request(socket_fd, epoll_fd, unauthenticated_connections)?;
        }
    };
    Ok(())
}

fn handle_new_connection_request(
    socket_fd: i32,
    epoll_fd: i32,
    unauthenticated_connections: &mut Vec<i32>,
) -> Result<()> {
    let connection_fd = unsafe { accept4(socket_fd, null_mut(), null_mut(), SOCK_NONBLOCK) };
    if connection_fd == -1 {
        bail!("could not accept a connection request: {}", Error::last_os_error());
    }
    register_interest(
        epoll_fd,
        connection_fd,
        &mut new_epoll_event(EPOLLIN, connection_fd),
    )?;
    unauthenticated_connections.push(connection_fd);
    println!("accepted a connection with fd: {}", connection_fd);
    Ok(())
}

fn handle_data_on_connection(
    connection_fd: i32,
    open_connections: &mut HashMap<i32, String>,
    epoll_fd: i32,
) -> Result<String> {
    //this event concerns a connection: either FIN or regular data
    let mut buf: [u8; 256] = [0; 256];
    //make sure to drain completely
    let first_read_response = read_connection_fd(&mut buf, connection_fd);
    if first_read_response == 0 {
        //EOF, CLIENT SENT A FIN
        println!("the connection {connection_fd} sent a FIN. Removing and closing it");
        open_connections.remove(&connection_fd);
        close_connection(connection_fd, epoll_fd)?;
        bail!("0")
    } else if first_read_response == -1 {
        //most likely: it's a blockin call prevented, try again later. might be a genuine error in some cases
        println!("error reading connection {connection_fd}");
        open_connections.remove(&connection_fd);
        close_connection(connection_fd, epoll_fd)?;
        bail!("-1")
    } else {
        //everythin was fine
        println!("connection {connection_fd} sent some data");
        //try to convert to utf-8
        let text = String::from_utf8(buf.to_vec())?;
        Ok(text)
    }
}


fn read_connection_fd(buf: &mut [u8; 256], concerned_fd: i32) -> isize {
    let mut first_read_response: Option<isize> = None;
    loop {
        //VERY IMP: READ ERROR DOES NOT NECESSERILIY MEAN ERROR, IT JUST MEANS THERE IS NO MORE DATA **YET**
        let read_response = unsafe { read(concerned_fd, buf.as_mut_ptr() as *mut _, buf.len()) };
        let is_noncarryable_error =
            read_response == -1 && Error::last_os_error().kind() != ErrorKind::WouldBlock;
        if first_read_response.is_none() || is_noncarryable_error {
            //why the second condition ? because if its a genuine error, no point in continuing
            first_read_response = Some(read_response)
        }
        if read_response <= 0 {
            //EOF or ERROR, no need to continue either way
            break;
        }
    }
    first_read_response.unwrap()
}

fn close_connection(
    connection_fd: i32,
    epoll_fd: i32,
) -> Result<()> {
    unregister_interest(
        epoll_fd,
        connection_fd,
        &mut new_epoll_event(EPOLLIN, connection_fd),
    )?;
    unsafe { close(connection_fd) };
    Ok(())
}
