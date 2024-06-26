use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io;

use super::{incus_output, Location};

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
/// Incus image information
pub struct Image {
    pub auto_update: bool,
    pub properties: BTreeMap<String, String>,
    pub public: bool,
    pub aliases: Vec<BTreeMap<String, String>>,
    pub architecture: String,
    pub cached: bool,
    pub filename: String,
    pub fingerprint: String,
    pub size: u64,
    #[serde(default = "BTreeMap::new")]
    pub update_source: BTreeMap<String, String>,
    pub created_at: String,
    pub expires_at: String,
    pub last_used_at: String,
    pub uploaded_at: String,
}

impl Image {
    /// Retrieve Incus container image information from all images
    ///
    /// # Arguments
    ///
    /// * `location` - The location of the host
    ///
    /// # Return
    ///
    /// The Incus image information
    ///
    /// # Errors
    ///
    /// Errors that are encountered while retrieving image info will be returned
    ///
    /// # Example
    ///
    /// ```
    /// use incus::{Image, Location};
    ///
    /// let images = Image::all(Location::Local).unwrap();
    /// ```
    pub fn all(location: Location) -> io::Result<Vec<Self>> {
        let json = match location {
            Location::Local => incus_output(&["image", "list", "--format", "json"])?,
            Location::Remote(remote) => {
                incus_output(&["image", "list", &format!("{}:", remote), "--format", "json"])?
            }
        };

        serde_json::from_slice::<Vec<Self>>(&json).map_err(|err| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Incus image: failed to parse json: {}", err),
            )
        })
    }

    /// Retrieve Incus image information from one image
    ///
    /// # Arguments
    ///
    /// * `location` - The location of the host
    /// * `name` - The name of the container
    ///
    /// # Return
    ///
    /// The Incus image information
    ///
    /// # Errors
    ///
    /// Errors that are encountered while retrieving image info will be returned
    /// ```
    pub fn new(location: Location, name: &str) -> io::Result<Self> {
        let json = match location {
            Location::Local => incus_output(&["image", "list", name, "--format", "json"])?,
            Location::Remote(remote) => incus_output(&[
                "image",
                "list",
                &format!("{}:", remote),
                name,
                "--format",
                "json",
            ])?,
        };

        match serde_json::from_slice::<Vec<Self>>(&json) {
            Ok(mut list) => {
                if list.len() == 1 {
                    Ok(list.remove(0))
                } else {
                    Err(io::Error::new(
                        io::ErrorKind::NotFound,
                        format!("Incus image: {} not found", name),
                    ))
                }
            }
            Err(err) => Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Incus image: failed to parse json: {}", err),
            )),
        }
    }
}
