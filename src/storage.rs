use serde::Serialize;
use std::{fs, path};

use sha2::{Digest, Sha256};

use crate::{CheckResult, LibError};

/// Storage

pub struct CheckResultStorage {
    path: path::PathBuf,
}

fn get_sha256_string<T: Serialize>(value: &T) -> Result<String, LibError> {
    // FIXME: use something else than json for serializing to a hash, that is stupidly inefficient.
    let json = serde_json::to_string(&value).map_err(|source| LibError::JsonError { source })?;
    // FIXME: isn't there a convenience function for hashing ?
    let mut hasher = Sha256::new();
    hasher.update(json);
    let result = hasher.finalize();
    Ok(format!("{result:x}"))
}

impl CheckResultStorage {
    /// Builds a new storage
    pub fn new(path: &path::PathBuf) -> Result<Self, LibError> {
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
        let hash = get_sha256_string(servers)?;
        let file_name = format!("{provider_name}-{hash}.sha256");
        // FIXME: optimize the building of path instead of mutable things ?
        let mut path = self.path.clone();
        path.push("check_result");
        path.push(file_name);
        Ok(dbg!(path))
    }

    /// Stores the hash of a provided provider/servers combo
    pub fn put_hash(
        &self,
        provider_name: &str,
        servers: &Vec<String>,
        check_result: &CheckResult,
    ) -> Result<(), LibError> {
        let path = self.get_path(&provider_name, &servers)?;
        let available_server_hash = get_sha256_string(&check_result.available_servers)?;
        fs::write(path, dbg!(available_server_hash)).map_err(|source| LibError::IOError { source })
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
    ///
    /// Example:
    /// ```
    /// match self.get_check_result_hash(provider_name, servers)? { // Err on critical
    ///   None => Ok(false),                                        // file not found
    ///   Some(stored_hash) => Ok(true),                            // string read and trimmed
    /// }
    /// ```
    pub fn get_hash(
        &self,
        provider_name: &str,
        servers: &Vec<String>,
    ) -> Result<Option<String>, LibError> {
        // not being able to build the file path is a problem, so we might return an Err
        let path = self.get_path(&provider_name, &servers)?;
        // handle the result of reading the file as a textual string
        match dbg!(fs::read_to_string(path)) {
            Err(err) => match err.kind() {
                // not being able to read the file IF IT DOES NOT EXIST is NOT a problem.
                std::io::ErrorKind::NotFound => Ok(None),
                // any other reason we could not get a string IS a problem.
                _ => Err(LibError::IOError { source: err }),
            },
            // if the string was read successfully, trim it to remove any whitespace and newlines
            Ok(content) => Ok(Some(content.trim().to_string())),
        }
    }

    /// Compares the provided check_result by building its hash and comparing to the one stored
    /// Its error behaviour is the same as `get_check_result_hash()`
    pub fn is_equal(
        &self,
        provider_name: &str,
        servers: &Vec<String>,
        check_result: &CheckResult,
    ) -> Result<bool, LibError> {
        // get eventual stored string or produce an error if something critical happened
        let hash = self.get_hash(provider_name, servers)?;
        match hash {
            // by design, if the hash was not present on disk, check_result is deemed not equal
            None => Ok(false),
            // otherwise, compute the current check_result and compare it to the stored one
            Some(stored_hash) => {
                let available_server_hash =
                    dbg!(get_sha256_string(&check_result.available_servers))?;
                Ok(available_server_hash == stored_hash)
            }
        }
    }
}
