use std::io;
use std::path::Path;

use super::{incus, Location, Snapshot};

/// An Incus ephemeral container
pub struct Container {
    name: String,
}

impl Container {
    /// Create a new Incus container
    ///
    /// # Arguments
    ///
    /// * `location` - The location of the host
    /// * `name` - The name of the container
    /// * `base` - The base distribution to use, `ubuntu:16.04` for example
    ///
    /// # Return
    ///
    /// The newly created Incus container
    ///
    /// # Errors
    ///
    /// Errors that are encountered while creating container will be returned
    ///
    /// # Example
    ///
    /// ```
    /// use incus::{Container, Location};
    ///
    /// let mut container = Container::new(Location::Local, "test-new", "ubuntu:16.04").unwrap();
    /// ```
    pub fn new(location: Location, name: &str, base: &str) -> io::Result<Self> {
        let full_name = match location {
            Location::Local => name.to_string(),
            Location::Remote(remote) => format!("{}:{}", remote, name),
        };

        let image_base = format!("images:{base}");

        incus(&["launch", &image_base, &full_name, "-e", "-n", "incusbr0"])?;

        // XXX: https://bugzilla.redhat.com/show_bug.cgi?id=1419315
        incus(&[
            "exec",
            &full_name,
            "--mode=non-interactive",
            "-n",
            "--",
            "touch",
            "/etc/fstab",
        ])?;

        // Hack to wait for network up and running
        //incus(&[
        //    "exec",
        //    &full_name,
        //    "--mode=non-interactive",
        //    "-n",
        //    "--",
        //    "dhclient",
        //])?;

        // Start the container
        //incus(&["start", &full_name])?;

        Ok(Container { name: full_name })
    }

    /// Create a new privileged Incus container
    ///
    /// # Arguments
    ///
    /// * `location` - The location of the host
    /// * `name` - The name of the container
    /// * `base` - The base distribution to use, `ubuntu:16.04` for example
    ///
    /// # Return
    ///
    /// The newly created Incus container
    ///
    /// # Errors
    ///
    /// Errors that are encountered while creating container will be returned
    ///
    /// # Example
    ///
    /// ```
    /// use incus::{Container, Location};
    ///
    /// let mut container = unsafe { Container::new_privileged(Location::Local, "test-new-privileged", "ubuntu:16.04").unwrap() };
    /// ```
    pub unsafe fn new_privileged(location: Location, name: &str, base: &str) -> io::Result<Self> {
        let full_name = match location {
            Location::Local => name.to_string(),
            Location::Remote(remote) => format!("{}:{}", remote, name),
        };

        incus(&[
            "launch",
            base,
            &full_name,
            "-e",
            "-n",
            "incusbr0",
            "-c",
            "security.privileged=true",
            "-c",
            "raw.incus=incus.apparmor.profile=unconfined",
        ])?;

        // XXX: https://bugzilla.redhat.com/show_bug.cgi?id=1419315
        incus(&[
            "exec",
            &full_name,
            "--mode=non-interactive",
            "-n",
            "--",
            "touch",
            "/etc/fstab",
        ])?;

        // Hack to wait for network up and running
        incus(&[
            "exec",
            &full_name,
            "--mode=non-interactive",
            "-n",
            "--",
            "dhclient",
        ])?;

        Ok(Container { name: full_name })
    }

    /// Get full name of container
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Create a snapshot of a container
    ///
    /// # Arguments
    ///
    /// * `name` - name of the new snapshot
    ///
    /// # Return
    ///
    /// A new snapshot on success
    ///
    /// # Errors
    ///
    /// Errors that are encountered while creating snapshot will be returned
    ///
    /// # Example
    ///
    /// ```
    /// use incus::{Container, Location, Snapshot};
    ///
    /// let container = Container::new(Location::Local, "test-snapshot", "ubuntu:16.04").unwrap();
    /// container.snapshot("test-snapshot").unwrap();
    /// ```
    pub fn snapshot<'a>(&'a self, name: &str) -> io::Result<Snapshot<'a>> {
        Snapshot::new(self, name)
    }

    /// Run a command in an Incus container
    ///
    /// # Arguments
    ///
    /// * `command` - An array of command arguments
    ///
    /// # Return
    ///
    /// An empty tuple on success
    ///
    /// # Errors
    ///
    /// Errors that are encountered while executing will be returned
    ///
    /// # Example
    ///
    /// ```
    /// use incus::{Container, Location};
    ///
    /// let mut container = Container::new(Location::Local, "test-exec", "ubuntu:16.04").unwrap();
    /// container.exec(&["echo", "hello"]).unwrap();
    /// ```
    pub fn exec(&mut self, command: &[&str]) -> io::Result<()> {
        let mut args = vec!["exec", &self.name, "--"];
        for arg in command.as_ref().iter() {
            args.push(arg.as_ref());
        }
        incus(&args)
    }

    /// Add a profile to an Incus container
    ///
    /// # Arguments
    ///
    /// * `profile_name` - A string of the profile name
    ///
    /// # Return
    ///
    /// An empty tuple on success
    ///
    /// # Errors
    ///
    /// Errors that are encountered while executing will be returned
    ///
    /// # Example
    ///
    /// ```
    /// use incus::{Container, Location};
    ///
    /// let mut container = Container::new(Location::Local, "test-exec", "ubuntu:16.04").unwrap();
    /// container.add_profile("default").unwrap();
    /// ```
    pub fn add_profile(&mut self, profile_name: &str) -> io::Result<()> {
        let args = vec!["profile", "add", &self.name, profile_name];
        incus(&args)
    }

    /// Mount a path in an Incus container
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the mount
    /// * `source` - The source path to mount
    /// * `dest` - The destination of the mount
    ///
    /// # Return
    ///
    /// An empty tuple on success
    ///
    /// # Errors
    ///
    /// Errors that are encountered while mounting will be returned
    ///
    /// # Example
    ///
    /// ```
    /// use incus::{Container, Location};
    ///
    /// let mut container = Container::new(Location::Local, "test-mount", "ubuntu:16.04").unwrap();
    /// container.mount("source", ".", "/root/source").unwrap();
    /// ```
    pub fn mount<P: AsRef<Path>>(&mut self, name: &str, source: P, dest: &str) -> io::Result<()> {
        incus(&[
            "config",
            "device",
            "add",
            &self.name,
            name,
            "disk",
            &format!("source={}", source.as_ref().display()),
            &format!("path={}", dest),
        ])
    }

    /// Push a file to the Incus container
    ///
    /// # Arguments
    ///
    /// * `source` - The source of the file in the host
    /// * `dest` - The destination of the file in the container
    /// * `recursive` - The source is a directory
    ///
    /// # Return
    ///
    /// An empty tuple on success
    ///
    /// # Errors
    ///
    /// Errors that are encountered while pushing will be returned
    ///
    /// # Example
    ///
    /// ```
    /// extern crate incus;
    /// extern crate tempdir;
    ///
    /// use incus::{Container, Location};
    /// use tempdir::TempDir;
    ///
    /// fn main() {
    ///     let mut container = Container::new(Location::Local, "test-push", "ubuntu:16.04").unwrap();
    ///     let tmp = TempDir::new("").unwrap();
    ///     container.push(tmp.path(), "/root", true).unwrap();
    /// }
    /// ```
    pub fn push<P: AsRef<Path>>(
        &mut self,
        source: P,
        dest: &str,
        recursive: bool,
    ) -> io::Result<()> {
        if recursive {
            incus(&[
                "file",
                "push",
                "--quiet",
                "--recursive",
                &format!("{}", source.as_ref().display()),
                &format!("{}/{}", self.name, dest),
            ])
        } else {
            incus(&[
                "file",
                "push",
                "--quiet",
                &format!("{}", source.as_ref().display()),
                &format!("{}/{}", self.name, dest),
            ])
        }
    }

    /// Pull a file from the Incus container
    ///
    /// # Arguments
    ///
    /// * `source` - The source of the file in the container
    /// * `dest` - The destination of the file in the host
    /// * `recursive` - The source is a directory
    ///
    /// # Return
    ///
    /// An empty tuple on success
    ///
    /// # Errors
    ///
    /// Errors that are encountered while pulling will be returned
    ///
    /// # Example
    ///
    /// ```
    /// extern crate incus;
    /// extern crate tempdir;
    ///
    /// use incus::{Container, Location};
    /// use tempdir::TempDir;
    ///
    /// fn main() {
    ///     let mut container = Container::new(Location::Local, "test-pull", "ubuntu:16.04").unwrap();
    ///     container.exec(&["mkdir", "artifacts"]).unwrap();
    ///     let tmp = TempDir::new("").unwrap();
    ///     container.pull("/root/artifacts", tmp.path(), true).unwrap();
    /// }
    /// ```
    pub fn pull<P: AsRef<Path>>(
        &mut self,
        source: &str,
        dest: P,
        recursive: bool,
    ) -> io::Result<()> {
        if recursive {
            incus(&[
                "file",
                "pull",
                "--quiet",
                "--recursive",
                &format!("{}/{}", self.name, source),
                &format!("{}", dest.as_ref().display()),
            ])
        } else {
            incus(&[
                "file",
                "pull",
                "--quiet",
                &format!("{}/{}", self.name, source),
                &format!("{}", dest.as_ref().display()),
            ])
        }
    }
}

impl Drop for Container {
    fn drop(&mut self) {
        let _ = incus(&["stop", &self.name]);
    }
}
