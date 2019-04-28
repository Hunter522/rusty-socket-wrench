//! Combines Stdin and Stdout
//!
//! This only extends the Read and Write traits

use std::io::{self, Stdin, Stdout, StdinLock, StdoutLock, Read, Write};
use std::fmt::Arguments;

/// Combination of std::Stdin and std::Stdout
pub struct Stdio {
    pub stdin: Stdin,
    pub stdout: Stdout 
}

impl Stdio {

    /// Create a new instance of Stdio
    pub fn new() -> Stdio {
        Stdio {stdin: io::stdin(), stdout: io::stdout()}
    }

    // lock

    // read_line
}

impl Read for Stdio {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.stdin.read(buf)
    }
}

impl Write for Stdio {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.stdout.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.stdout.flush()
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.stdout.write_all(buf)
    }

    fn write_fmt(&mut self, fmt: Arguments) -> io::Result<()> {
        self.stdout.write_fmt(fmt)
    }
}
