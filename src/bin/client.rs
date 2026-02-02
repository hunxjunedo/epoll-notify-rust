use std::{io::{Write}, net::TcpStream, thread, time::Duration};

use anyhow::Result;

fn main() -> Result<()>{
    //initialize the connection
    let mut connection = TcpStream::connect("127.0.0.1:8080").unwrap();
    let mut connection2 = TcpStream::connect("127.0.0.1:8080").unwrap();
    connection.set_nodelay(true)?;
    connection2.set_nodelay(true)?;
    connection.write_all(b"hunx1")?;
    connection2.write_all(b"hunx1")?;
    thread::sleep(Duration::from_secs(10));
    connection.write_all(b"hunx2")?;
    connection2.write_all(b"hunx2")?;
    //CONFIRMED: does not send FIN until process exits
    Ok(())
}