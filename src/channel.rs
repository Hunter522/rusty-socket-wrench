//! The Channel trait is anything that can perform I/O operatinons
//! 
//! Supported channels:
//! * TCP Server
//! * TCP Client
//! * UDP Server
//! * UDP Client
//! * Serial port
//! * stdin/stdout

use std::net::{UdpSocket, TcpListener, TcpStream};
use std::io::{self, Write, Read, Stdin, Stdout, Error, ErrorKind};
use stdio::Stdio;
use std::os::unix::io::AsRawFd;

pub enum ChannelKind {
    Udp(UdpSocket),
    TcpServer(TcpListener),
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
                return channel_kind.send(buf);
            },
            ChannelKind::TcpServer(ref mut channel_kind) => {
                // cant just send...we have to listen first for a client to connect before we can send to it (TcpStream)
                // can either maintain a special-case vector of TcpStream representing any connected client
                // the main loop will have to special-case it to listen for any tcp clients
                // have to remove any tcp clients too that disconnect or close the stream
                unimplemented!()
            },
            ChannelKind::TcpClient(ref mut channel_kind) => {
                return channel_kind.write(buf);
            },
            ChannelKind::Stdio(ref mut channel_kind) => {
                return channel_kind.write(buf);
            }
        }
    }

    pub fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let result: io::Result<usize> = match self.channel_kind {
            ChannelKind::Udp(ref mut channel_kind) => {
                channel_kind.recv(buf)
            },
            ChannelKind::TcpClient(ref mut channel_kind) => {
                channel_kind.read(buf)
            },
            ChannelKind::TcpServer(ref mut channel_kind) => {
                unimplemented!()
            },
            ChannelKind::Stdio(ref mut channel_kind) => {
                channel_kind.read(buf)
            }
        };

        match result {
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                return Ok(0) // no data available, IS OK
            },
            _ => return result,
        }
    }
}

impl AsRawFd for Channel {
    fn as_raw_fd(&self) -> std::os::unix::io::RawFd {
        match self.channel_kind {
            ChannelKind::Udp(ref channel_kind) => {
                return channel_kind.as_raw_fd();
            },
            ChannelKind::TcpServer(ref channel_kind) => {
                return channel_kind.as_raw_fd();
            },
            ChannelKind::TcpClient(ref channel_kind) => {
                return channel_kind.as_raw_fd();
            },
            ChannelKind::Stdio(ref channel_kind) => {
                return channel_kind.stdin.as_raw_fd();
            }
        }
    }
}