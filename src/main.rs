#[macro_use]
extern crate log;
extern crate env_logger;
extern crate clap;
extern crate libc;

mod stdio;
mod tcp_server_wrapper;
mod channel;

use std::net::{UdpSocket, TcpStream};
use std::io::{self};
use clap::{Arg, App};
use stdio::Stdio;
use tcp_server_wrapper::TcpServerWrapper;
use channel::{Channel, ChannelKind};


const READ_BUF_SIZE: usize = 2048;
// const WRITE_BUF_SIZE: usize = 2048;


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
            let port: u16 = channel_params.parse().expect("Invalid channel params. Must be PORT");
            let addr = format!("0.0.0.0:{}", port);
            let mut socket = UdpSocket::bind(addr).expect("Couldn't bind to address");
            socket.set_nonblocking(true).expect("Failed to set nonblocking mode on socket");
            Ok(Channel::new(ChannelKind::Udp(socket)))
        },
        "udpout" => {
            debug!("udpout");
            // 0.0.0.0:0 is used to allow the UDP socket to be bound to any available port
            let mut socket = UdpSocket::bind("0.0.0.0:0").expect("Couldn't bind socket");
            socket.connect(channel_params).expect("Could not connect to address");
            socket.set_nonblocking(true).expect("Failed to set nonblocking mode on socket");
            Ok(Channel::new(ChannelKind::Udp(socket)))
        },
        "tcpin" => {
            debug!("tcpin");
            let mut socket = TcpServerWrapper::bind(channel_params).expect("Could not bind to address");
            Ok(Channel::new(ChannelKind::TcpServer(socket)))
        }
        "tcpout" => {
            debug!("tcpout");
            let mut socket = TcpStream::connect(channel_params).expect("Could not connect to address");
            socket.set_nonblocking(true).expect("Failed to set nonblocking mode on socket");
            Ok(Channel::new(ChannelKind::TcpClient(socket)))
        }
        _ => {
            // invalid channel type
            Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid channel type"))
        }
    }
}

#[allow(clippy::cyclomatic_complexity)]
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
    // let mut write_buf : [u8; WRITE_BUF_SIZE] = [0; WRITE_BUF_SIZE];


    // main loop
    loop {
        // special case for TcpServer: call accept()
        if let ChannelKind::TcpServer(server) = &mut input_channel.channel_kind {
            match server.accept() {
                Ok((_, addr)) => debug!("New TCP client connected to input channel {}", addr),
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}, // do nothing, is ok since we are nonblocking
                Err(e) => panic!("Failed to call accept on input channel {:?}", e)
            }
        }
        if let ChannelKind::TcpServer(server) = &mut output_channel.channel_kind {
            match server.accept() {
                Ok((_, addr)) => debug!("New TCP client connected to output channel {}", addr),
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}, // do nothing, is ok since we are nonblocking
                Err(e) => panic!("Failed to call accept on output channel {:?}", e)
            }
        }
        
        // set up pollfd structs for calling libc::poll
        let mut pollfd_structs_vec: Vec<libc::pollfd> = Vec::new();
        for raw_fd in input_channel.raw_fds() {
            pollfd_structs_vec.push(libc::pollfd {
                                        fd: raw_fd,
                                        events: libc::POLLIN,
                                        revents: 0
                                    });
        }
        let num_fd_in = pollfd_structs_vec.len();
        for raw_fd in output_channel.raw_fds() {
            pollfd_structs_vec.push(libc::pollfd {
                                        fd: raw_fd,
                                        events: libc::POLLIN,
                                        revents: 0
                                    });
        }
        
        debug!("Polling to see if data is available to be read on either channel...");
        let data_available = match rpoll(pollfd_structs_vec.as_mut_slice(), 500) {
            -1 => panic!("Error occurred when poll() was called"), //TODO: use errno crate
            0 => false, // timed out,
            _ => true // positive number returned on success
        };

        if data_available {
            let (pollfd_structs_in, pollfd_structs_out) = pollfd_structs_vec.split_at(num_fd_in);
            let input_channel_has_data = pollfd_structs_in.iter().any(|&x| x.revents == libc::POLLIN);
            let output_channel_has_data = pollfd_structs_out.iter().any(|&x| x.revents == libc::POLLIN);
                        
            let mut bytes_read_in = 0;
            let mut bytes_read_out = 0;

            // read
            if input_channel_has_data {
                bytes_read_in = match input_channel.read(&mut read_buf_input_channel) {
                    Ok(bytes_read) => bytes_read,
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => 0,
                    Err(e) => panic!("Failed to read data from input channel {:?}", e)
                };
                debug!("input_channel: Read {} bytes", bytes_read_in);
            }
            if output_channel_has_data {
                bytes_read_out = match output_channel.read(&mut read_buf_output_channel) {
                    Ok(bytes_read) => bytes_read,
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => 0,
                    Err(e) => panic!("Failed to read data from output channel {:?}", e)
                };
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
