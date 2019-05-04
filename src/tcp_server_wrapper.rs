//! TcpServer wrapper
//! 
//! Intended to help manage the clients that connect to this server

use std::net::{UdpSocket, TcpListener, TcpStream, ToSocketAddrs, SocketAddr};
use std::io::{self, Write, Read, Stdin, Stdout, Error, ErrorKind};
use stdio::Stdio;
use std::os::unix::io::AsRawFd;

pub struct TcpServerWrapper {
    pub server: TcpListener,
    pub clients: Vec<TcpStream>
}

impl TcpServerWrapper {
    pub fn bind<A: ToSocketAddrs>(addr: A) -> io::Result<TcpServerWrapper> {
        let server = TcpListener::bind(addr)?;
        server.set_nonblocking(true)?;
        
        Ok(TcpServerWrapper { server: server, clients: vec![] })
    }

    /// Wraps the TcpListener::accept call
    /// 
    /// Since the TcpListener is set to non-blocking mode, this call will immmediately return. 
    /// If a new incoming connection has been received, then the new TcpStream is added to the clients vector, 
    /// and a mutable reference returned
    pub fn accept(&mut self) -> io::Result<(&mut TcpStream, SocketAddr)> {
        match self.server.accept() {
            Ok((client, addr)) => {
                client.set_nonblocking(true)?; // configure client to be nonblocking like rest of the i/o
                self.clients.push(client);
                Ok((self.clients.last_mut().unwrap(), addr))
            },
            Err(e) => Err(e)
        }
    }
}
