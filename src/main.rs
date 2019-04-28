#[macro_use]
extern crate log;
extern crate env_logger;
extern crate clap;
extern crate libc;

mod stdio;
mod channel;
// mod channel;

use std::net::{UdpSocket, TcpListener, TcpStream};
use std::io::{self, Write, Read, Stdin, Stdout, Error, ErrorKind};
use std::fmt::Arguments;
use clap::{Arg, App};
use stdio::Stdio;
use channel::{Channel, ChannelKind};
use std::os::unix::io::AsRawFd;


const READ_BUF_SIZE: usize = 2048;
const WRITE_BUF_SIZE: usize = 2048;


// trait Channel: Read + Write { }
// impl<T: Read + Write> Channel for T { }
// impl Channel for T where T: Read + Write { }


// cant implement traits that aren't ours on types that aren't ours...we have to own one of them in our crate

// impl Channel for UdpSocket {
//     fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
//         1
//     }
// }

// struct UdpSocketWrapper(UdpSocket);
// struct TcpListenerWrapper(TcpListener);
// struct TcpStreamWrapper(TcpStream);



/// Rust wrapper around libc::poll
fn rpoll(fds: &mut [libc::pollfd], timeout: libc::c_int) -> libc::c_int {
    unsafe {
        libc::poll(&mut fds[0] as *mut libc::pollfd, fds.len() as libc::nfds_t, timeout)
    }
}




/// Parses cmd line string for channel
/// 
/// "udpin:PORT"      - UDP server - UdpSocket
/// "udpout:IP:PORT"  - UDP client - UdpSocket
/// "tcpin:PORT"      - TCP server - TcpListener
/// "tcpout:IP:PORT"  - TCP client - TcpStream
/// "stdio"           - stdin/stdout Stdin/Stdout
/// 
/// Returns Some(Read+Write) if successful, else returns None
fn parse_channel_str(channel_str: &str) -> io::Result<Channel> {
    // special case: stdin/stdout
    if channel_str == "stdio" {
        debug!("stdio");
        return Ok(Channel::new(ChannelKind::Stdio(Stdio::new())));
    }

    // everything else follows pattern "channel_type:channel_params"

    let idx = channel_str.find(':').expect("Invalid channel format: Missing ':'");
    let (channel_type, channel_params1) = channel_str.split_at(idx);
    let channel_params = &channel_params1[1..];

    match channel_type {
        "udpin" => {
            debug!("udpin");
            debug!("{}", channel_params);
            let port: u16 = channel_params.parse().expect("Invalid channel params. Must be PORT");
            let addr = format!("0.0.0.0:{}", port);
            let mut socket = UdpSocket::bind(addr).expect("Couldn't bind to address");
            socket.set_nonblocking(true).expect("Failed to set nonblocking mode on socket");
            return Ok(Channel::new(ChannelKind::Udp(socket)))
        },
        "udpout" => {
            debug!("udpout");
            // let ip_port_vec: Vec<&str> = channel_params.split(':')
            // let port_idx = channel_params.find(':').expect("Invalid channel params format: Missing ':'");
            // let (ip, port) = channel_params.split_at(port_idx);
            // 0.0.0.0:0 is used to allow the UDP socket to be bound to any available port
            let mut socket = UdpSocket::bind("0.0.0.0:0").expect("Couldn't bind socket");
            socket.connect(channel_params).expect("Could not connect to address");
            socket.set_nonblocking(true).expect("Failed to set nonblocking mode on socket");
            return Ok(Channel::new(ChannelKind::Udp(socket)))
        },
        "tcpin" => {
            debug!("tcpin");
            // return Err(io::Error::new(ErrorKind::InvalidInput, "Not implemented"));
            unimplemented!();
        }
        "tcpout" => {
            debug!("tcpout");
            unimplemented!();
        }
        _ => {
            // invalid channel type
            return Err(io::Error::new(ErrorKind::InvalidInput, "Invalid channel type"));
        }
    };
}

fn main() {
    env_logger::init();

    debug!("Starting up...");

    // implementation should follow a single-threaded approach...make it simple!

    // parse command line arguments
    let matches = App::new("rusty-socket-wrench")
        .version("0.1.0")
        .author("Hunter N. Morgan <hunterm522@gmail.com>")
        .about("Command line network relay tool")
        .arg(Arg::with_name("INPUT")
            .help("Input channel")
            .required(true))
        .arg(Arg::with_name("OUTPUT")
            .help("Output channel")
            .required(true))
        .get_matches();

    let input_channel_str = matches.value_of("INPUT").unwrap();
    let output_channel_str = matches.value_of("OUTPUT").unwrap();

    // set up input channel
    let mut input_channel = parse_channel_str(input_channel_str).unwrap();

    // set up output channel
    let mut output_channel = parse_channel_str(output_channel_str).unwrap();

    /*
    main loop
        read ops
        read from input into input read buffer
        read from output into output read buffer

        write ops
        write to output from input read buffer
        write to input from output read buffer
    */

    let mut read_buf_input_channel : [u8; READ_BUF_SIZE] = [0; READ_BUF_SIZE];
    let mut read_buf_output_channel : [u8; READ_BUF_SIZE] = [0; READ_BUF_SIZE];
    let mut write_buf : [u8; WRITE_BUF_SIZE] = [0; WRITE_BUF_SIZE];


    // main loop
    loop {
        // everything should be non-blocking
        // add sleep at end for couple ms, if data coming across, then remove sleep...optimization to reduce taxing cpu

        //stdio has no way to do nonblocking reads so have to offload that to another thread
        // look into using Rust "channels" which is inter-thread comms

        // either could use libc::poll to block until one of the channels has data to read
        // or just do non-blocking read and if no bytes read or WOULDBLOCK occurs then just sleep for a tiny bit
        // ehh...poll is better

        //TODO: in case of TcpServer, either listen here or create a new thread that just does the listening for TcpClients
        //      can pass the fd to channel::read call...special case for tcpserver where it will use that argument

        //TODO: to work with TcpServer, since it has multiple TcpClients, would have to special case on that
        // and dynamically add all the TcpClient::as_raw_fd() 
        let mut pollfd_structs = [
            libc::pollfd {
                fd: input_channel.as_raw_fd(),
                events: libc::POLLIN,
                revents: 0
            },
            libc::pollfd {
                fd: output_channel.as_raw_fd(),
                events: libc::POLLIN,
                revents: 0
            }];
        
        debug!("Polling to see if data is available to be read on either channel...");
        let data_available = match rpoll(&mut pollfd_structs, 500) {
            -1 => panic!("Error occurred when poll() was called"), //TODO: use errno crate
            0 => false, // timed out,
            _ => true // positive number returned on success
        };

        if data_available {
            let input_channel_has_data = pollfd_structs[0].revents == libc::POLLIN;
            let output_channel_has_data = pollfd_structs[1].revents == libc::POLLIN;
            let mut bytes_read_in = 0;
            let mut bytes_read_out = 0;

            // read
            if input_channel_has_data {
                bytes_read_in = input_channel.read(&mut read_buf_input_channel).unwrap();
                debug!("input_channel: Read {} bytes", bytes_read_in);
            }
            if output_channel_has_data {
                bytes_read_out = output_channel.read(&mut read_buf_output_channel).unwrap();
                debug!("output_channel: Read {} bytes", bytes_read_out);
            }
        
            // write
            if bytes_read_in > 0 {
                let bytes_written_out = output_channel.write(&mut read_buf_input_channel[0..bytes_read_in]).unwrap();
                debug!("output_channel: Wrote {} bytes", bytes_written_out);
            }
            if bytes_read_out > 0 {
                let bytes_written_in = input_channel.write(&mut read_buf_output_channel[0..bytes_read_out]).unwrap();
                debug!("input_channel: Wrote {} bytes", bytes_written_in);
            }
        }
    }
}
