//! The Channel trait is anything that can perform I/O operatinons
//! 
//! Supported channels:
//! * TCP Server
//! * TCP Client
//! * UDP Server
//! * UDP Client
//! * Serial port
//! * stdin/stdout

use std::net::{UdpSocket, TcpStream};
use std::io::{self, Write, Read};
use std::os::unix::io::{RawFd, AsRawFd};
use stdio::Stdio;
use tcp_server_wrapper::TcpServerWrapper;

pub enum ChannelKind {
    Udp(UdpSocket),
    TcpServer(TcpServerWrapper),
    TcpClient(TcpStream),
    Stdio(Stdio)
}

pub struct Channel {
    pub channel_kind: ChannelKind
}

impl Channel {
    pub fn new(channel_kind: ChannelKind) -> Channel {
        Channel { channel_kind: channel_kind }
    }

    pub fn write(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self.channel_kind {
            ChannelKind::Udp(ref mut channel_kind) => {
                channel_kind.send(buf)
            },
            ChannelKind::TcpServer(ref mut channel_kind) => {
                // cant just send...we have to listen first for a client to connect before we can send to it (TcpStream)
                // can either maintain a special-case vector of TcpStream representing any connected client
                // the main loop will have to special-case it to listen for any tcp clients
                // have to remove any tcp clients too that disconnect or close the stream

                // sends to all connected clients
                for client in &mut channel_kind.clients {
                    client.write_all(buf)?;
                }
                Ok(buf.len())
            },
            ChannelKind::TcpClient(ref mut channel_kind) => {
                channel_kind.write_all(buf)?;
                Ok(buf.len())
            },
            ChannelKind::Stdio(ref mut channel_kind) => {
                channel_kind.write_all(buf)?;
                Ok(buf.len())
            }
        }
    }

    pub fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self.channel_kind {
            ChannelKind::Udp(ref mut channel_kind) => {
                let (bytes_read, addr) = channel_kind.recv_from(buf)?;
                channel_kind.connect(addr)?; // "connect" to the sender, so that send call works
                Ok(bytes_read)
            },
            ChannelKind::TcpClient(ref mut channel_kind) => {
                channel_kind.read(buf)
            },
            ChannelKind::TcpServer(ref mut channel_kind) => {
                // hard to multiplex here, since by default it would interleave sometimes depending on
                // when I do the read and if I read all data from any particular client
                // so for now this will concatentant all data read from each client into one buffer then pass it on
                let mut total_bytes_read: usize = 0;
                for client in &mut channel_kind.clients {
                    total_bytes_read += match client.read(buf) {
                        Ok(bytes_read) => bytes_read,
                        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => 0,
                        Err(e) => return Err(e) // bubble up
                    };
                }
                Ok(total_bytes_read)
            },
            ChannelKind::Stdio(ref mut channel_kind) => {
                channel_kind.read(buf)
            }
        }
    }

    pub fn raw_fds(&self) -> Vec<RawFd> {
        match self.channel_kind {
            ChannelKind::Udp(ref channel_kind) => {
                vec![channel_kind.as_raw_fd()]
            },
            ChannelKind::TcpServer(ref channel_kind) => {
                channel_kind.clients.iter().map(AsRawFd::as_raw_fd).collect()
            },
            ChannelKind::TcpClient(ref channel_kind) => {
                vec![channel_kind.as_raw_fd()]
            },
            ChannelKind::Stdio(ref channel_kind) => {
                vec![channel_kind.stdin.as_raw_fd()]
            }
        }
    }
}
