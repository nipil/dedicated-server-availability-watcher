use crate::{CheckResult, LibError};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{fs, path};
use tracing::{debug, instrument, trace, warn};

// Storage

/// Structure to access disk storage, and store CheckResult hashes
///
/// path: the base directory for relative storage
pub struct CheckResultStorage {
    path: path::PathBuf,
}

/// Generates a SHA256 hash-string of the argument
///
/// we use json serialization as an intermediary data, because
/// - we already include serde_json crate
/// - we mainly hash String and Vec<String> for which json is "good enough"
///
/// We use the convenience function for Sha256 as we work blocking and data is small
///
fn to_json_sha256<T: Serialize>(value: &T) -> Result<String, LibError> {
    let json = serde_json::to_string(&value).map_err(|source| LibError::JsonError { source })?;
    let hash = format!("{:x}", Sha256::digest(&json));
    trace!("to_json_sha256: {json:?} -> {hash:?}");
    Ok(hash)
}

impl CheckResultStorage {
    /// Builds a new storage
    pub fn new(path: &path::PathBuf) -> Result<Self, LibError> {
        fs::create_dir_all(&path).map_err(|err| return LibError::IOError { source: err })?;
        if !path.is_dir() {
            return Err(LibError::ValueError {
                name: "Storage directory is not an accessible directory".to_string(),
                value: path.to_string_lossy().to_string(),
            });
        }
        Ok(Self { path: path.into() })
    }

    /// Builds the storage path for a provided provider/servers combo
    fn get_path(
        &self,
        provider_name: &str,
        servers: &Vec<String>,
    ) -> Result<path::PathBuf, LibError> {
        let hash = to_json_sha256(servers)?;
        let file_name = format!("{provider_name}-{hash}.sha256");
        let mut path = self.path.clone();
        path.push(file_name);
        Ok(path)
    }

    /// Stores the hash of a provided provider/servers combo
    #[instrument(skip_all, level = "debug")]
    pub fn put_hash(
        &self,
        provider_name: &str,
        servers: &Vec<String>,
        check_result: &CheckResult,
    ) -> Result<(), LibError> {
        let path = self.get_path(&provider_name, &servers)?;
        let available_server_hash = to_json_sha256(&check_result.available_servers)?;
        debug!(
            "put_hash {available_server_hash} to {}",
            path.to_string_lossy()
        );
        fs::write(path, available_server_hash).map_err(|source| LibError::IOError { source })
    }

    /// Gets the hash of a provided provider/servers combo
    ///
    /// Returns an Err if it cannot read the string content of the underlying
    /// file for any other reason than the file does not exist.
    ///
    /// The reason an error might happen is :
    /// - not being to generate the filename from the provider/server combo
    /// - not having permission to read the underlying file
    /// - any kind of text encoding error while converting the content to a string
    ///
    /// Returns None if the file was simply not found
    ///
    /// Returns Some(String) if a string has been read successfully from the file
    #[instrument(skip_all, level = "debug")]
    pub fn get_hash(
        &self,
        provider_name: &str,
        servers: &Vec<String>,
    ) -> Result<Option<String>, LibError> {
        // not being able to build the file path is a problem, so we might return an error
        let path = self.get_path(&provider_name, &servers)?;
        // handle the result of reading the file as a textual string
        match fs::read_to_string(&path) {
            Err(err) => match err.kind() {
                // not being able to read the file IF IT DOES NOT EXIST is NOT a problem.
                std::io::ErrorKind::NotFound => Ok(None),
                // any other reason we could not get a string IS a problem.
                _ => Err(LibError::IOError { source: err }),
            },
            // if the string was read successfully, trim it to remove any whitespace and newlines
            Ok(content) => {
                let stored_available_server_hash = content.trim().to_string();
                debug!(
                    "get_hash {stored_available_server_hash} from {}",
                    path.to_string_lossy()
                );
                Ok(Some(stored_available_server_hash))
            }
        }
    }

    /// Compares the provided check_result by building its hash and comparing to the one stored
    /// Its error behaviour is the same as `get_hash()`
    pub fn is_equal(
        &self,
        provider_name: &str,
        servers: &Vec<String>,
        check_result: &CheckResult,
    ) -> Result<bool, LibError> {
        // get eventual stored string or produce an error if something critical happened
        let stored_hash = self.get_hash(provider_name, servers)?;
        match stored_hash {
            // by design, if the hash was not present on disk, check_result is deemed not equal
            None => Ok(false),
            // otherwise, compute the current check_result and compare it to the stored one
            Some(stored_hash) => {
                let available_server_hash = to_json_sha256(&check_result.available_servers)?;
                Ok(available_server_hash == stored_hash)
            }
        }
    }
}
