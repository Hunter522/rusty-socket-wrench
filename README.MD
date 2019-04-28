# Rusty Socket Wrench

rusty-socket-wrench is a simple command-line tool to relay bidirectional data between two data channels.

Data channels can be TCP server, TCP client, UDP server, UDP client, serial port, or even stdin/stdout in many different configurations.

This was inspired by typical Linux tools such as `netcat` and `socat` but to provide a easier to use command-line interface.

The philosophy behind this tool is to provide sane defaults to sockets and not having any "surprises" in its behavior.


Some important behavior:

* tcp clients should try to attempt to connect constantly (make option not to)
* tcp cleints should attempt to reconnect if loss connection
* tcp clients should be able to detect loss of connection fairly quickly
* please please log what is going on!
* 

# TODO

- [ ] tcp server channel needs to be wrapper class to manage clients
- [ ] tcpin
- [ ] tcpout
- [ ] udpin should cache addr:port of sender so we know who to send to
- [ ] serial port


# Build

## Dependencies

```
log = "0.4.6"
env_logger = "0.6.0"
clap = "2.32.0"
libc = "0.2.47"
```

## Build procedure

# Install

# Usage

```
rusty-socket-wrench [-h] [GENERAL_OPTIONS] [INPUT_OPTIONS] [INPUT] [OUTPUT_OPTIONS] [OUTPUT]

*GENERAL_OPTIONS*
-unidirectional
-verbosity n
-



*INPUT_OPTIONS and OUTPUT_OPTIONS*
INPUT_OPTIONS and OUTPUT_OPTIONS are differentiated with a prefix "i" or "o". Example: "-itcpmaxclients 5" vs "-otcpmaxclients 5".
Some options are prefixed with "udp" or "tcp" so as to easily identify which type of channel they are options for.
-tcpmaxclients
-recvbufsize
-udpdatagramsize - if relaying from streaming channel (e.g. tcp socket), then this specifies the number of bytes to receive first from the other end before sending a UDP datagram. 

*INPUT AND OUTPUT*
INPUT and OUTPUT are the same
INPUT/OUTPUT formats:
"udpin:PORT"      - UDP server
"udoout:IP:PORT"  - UDP client
"tcpin:PORT"      - TCP server
"tcpout:IP:PORT"  - TCP client
"serial:PORT      - serial port
"-"               - stdin/stdout

```