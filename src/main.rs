//! High-performance /sbin/init program for Linux
//!
//! This is designed to do literally nothing but accept binaries over the
//! network and run them as a child of init.
//!
//! If you pipe a file to `<server ip>:1234` it will run it and pipe the
//! stderr and stdout back to you
//!
//! If you connect to `<server ip>:1235` init will send SIGKILL to all
//! processes on the system but itself. This is a measure to allow resetting
//! the system if a binary was uploaded that had issues. This port neither
//! sends or recieves anything, it simply kills upon getting a TCP connection.
//!
//! For a simple headless Linux machine running a basic kernel, you'll want
//! flags like this:
//!
//! console=ttyS1,115200 rw root=/dev/sda ip=dhcp
//!
//! This enables a console on ttyS1 (in my case that's COM2, the
//! Serial-over-LAN port for IPMI), `rw` specifies that the root mount should
//! be read-writable (required since we drop a file), root specifies the root
//! filesystem device (in our case we used an unpartioned flash drive with
//! vfat), and `ip=dhcp` is the coolest part, this allows the kernel to get
//! a DHCP lease on any active NICs. This is mandatory because we use
//! networking in `init` without any configuration of the network.

use std::io::{Read, Write};
use std::os::fd::{OwnedFd, FromRawFd, IntoRawFd};
use std::net::{TcpStream, TcpListener};
use std::error::Error;
use std::process::Command;

/// Worker for downloading and running a file
fn worker(stream: std::io::Result<TcpStream>)
        -> Result<(), Box<dyn Error>> {
    let mut stream = stream?;

    {
        // Read the file
        let mut buf = Vec::with_capacity(16 * 1024 * 1024);
        stream.read_to_end(&mut buf)?;

        // Write the file
        std::fs::write("/deez_bytes", &buf)?;

        println!("Wrote file with {} bytes", buf.len());
    }

    // Execute the file
    let mut result = Command::new("/deez_bytes")
        .stderr(unsafe {
            OwnedFd::from_raw_fd(stream.try_clone()?.into_raw_fd())
        })
        .stdout(unsafe {
            OwnedFd::from_raw_fd(stream.try_clone()?.into_raw_fd())
        })
        .spawn()?;

    // Wait for the process to exit
    let status = result.wait()?;

    // Pretty print the status
    let info = format!("Command returned {status}\n");

    // Log the status
    print!("{info}");

    // Send the end-user the result
    stream.write_all(info.as_bytes())?;

    Ok(())
}

fn main() {
    std::thread::spawn(|| {
        // Kill thread, if this gets a connection, it will kill any running
        // processes
        let listener = TcpListener::bind("0.0.0.0:1235")
            .expect("Could not bind to 1235");
        for stream in listener.incoming() {
            // Kill all processes but ourself
            unsafe { libc::kill(-1, libc::SIGKILL); }

            if let Ok(mut stream) = stream {
                let _ = stream.write_all(b"Kill command received, \
                    sent SIGKILL to all processes\n");
            }
        }
    });

    let listener = TcpListener::bind("0.0.0.0:1234")
        .expect("Could not bind to 1234");
    for stream in listener.incoming() {
        if let Err(err) = worker(stream) {
            eprintln!("worker returned error {:?}", err);
        }
    }

    unsafe {
        libc::reboot(libc::LINUX_REBOOT_CMD_RESTART);
    }
}

