//! A Rust library for controlling Incus

use std::io;
use std::process::{Command, Stdio};

pub use container::Container;
pub use image::Image;
pub use info::Info;
pub use location::Location;
pub use snapshot::Snapshot;

mod container;
mod image;
mod info;
mod location;
mod snapshot;

fn incus(args: &[&str]) -> io::Result<()> {
    let mut cmd = Command::new("incus");
    for arg in args.iter() {
        cmd.arg(arg);
    }

    let status = cmd.spawn()?.wait()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Incus {:?} failed with {}", args, status),
        ))
    }
}

fn incus_output(args: &[&str]) -> io::Result<Vec<u8>> {
    let mut cmd = Command::new("incus");
    for arg in args.iter() {
        cmd.arg(arg);
    }
    cmd.stdout(Stdio::piped());

    let output = cmd.spawn()?.wait_with_output()?;
    if output.status.success() {
        Ok(output.stdout)
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Incus {:?} failed with {}", args, output.status),
        ))
    }
}
