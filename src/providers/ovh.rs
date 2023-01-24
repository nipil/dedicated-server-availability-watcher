use std::env;

use reqwest::blocking::Response;
use serde::Deserialize;

use crate::LibError;

use super::{ProviderFactoryTrait, ProviderTrait, ServerInfo};

// OVH implementation

/// Common name to identify the provider
pub const OVH_NAME: &str = "ovh";

/// Common environment variable to eventually filter the queries.
const ENV_NAME_OVH_EXCLUDE_DATACENTER: &str = "OVH_EXCLUDE_DATACENTER";

/// Provider API endpoint.
const OVH_URL: &str = "https://api.ovh.com/1.0/dedicated/server/datacenter/availabilities";

/// Used for API result deserialisation.
#[derive(Deserialize)]
struct OvhDedicatedServerInformation {
    datacenters: Vec<OvhDedicatedServerDatacenterAvailability>,
    memory: Option<String>,
    storage: Option<String>,
    server: String,
}

/// Used for API result deserialisation.
impl OvhDedicatedServerInformation {
    fn is_available(&self) -> bool {
        for datacenter in self.datacenters.iter() {
            if datacenter.is_available() {
                return true;
            }
        }
        return false;
    }
}

/// Used for API result deserialisation.
#[derive(Deserialize)]
struct OvhDedicatedServerDatacenterAvailability {
    availability: String,
}

impl OvhDedicatedServerDatacenterAvailability {
    /// Evaluates availability.
    fn is_available(&self) -> bool {
        match self.availability.as_str() {
            "unavailable" => return false,
            "unknown" => return false,
            _ => return true,
        }
    }
}

/// Gets server inventory and availability.
pub struct Ovh {
    /// Used to exclude datacenters by their id.
    /// Examples : ca,bhs,fr,gra,rbx,sbg
    excluded_datacenters: Option<String>,
}

impl Ovh {
    /// Builds a new instance.
    fn new() -> Self {
        let p = env::var(ENV_NAME_OVH_EXCLUDE_DATACENTER)
            .unwrap_or_default()
            .trim()
            .to_string();
        Ovh {
            excluded_datacenters: if p.is_empty() { None } else { Some(p) },
        }
    }

    /// Gets availability for specified server types.
    /// `server`: optionally used to query for a single server type.
    fn query_available_servers(&self, server: Option<&str>) -> Result<Response, LibError> {
        let mut query: Vec<(&str, &str)> = Vec::new();

        match &self.excluded_datacenters {
            None => {
                query.push(("excludeDatacenters", "false"));
            }
            Some(excluded_datacenters) => {
                query.push(("excludeDatacenters", "true"));
                query.push(("datacenters", &excluded_datacenters));
            }
        }

        // Handles optional filtering.
        if let Some(server) = server {
            query.push(("server", server));
        }

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(OVH_URL)
            .query(&query)
            .send()
            .map_err(|source| LibError::RequestError { source })?;

        if !response.status().is_success() {
            return Err(LibError::ApiError {
                message: format!("Error during OVH query: code {}", response.status()),
            });
        }

        Ok(response)
    }
}

impl ProviderFactoryTrait for Ovh {
    /// Builds an Ovh provider from environment variables.
    fn from_env() -> Result<Box<dyn ProviderTrait>, LibError> {
        Ok(Box::new(Ovh::new()))
    }
}

impl ProviderTrait for Ovh {
    /// Gets the actual name of the provider.
    fn name(&self) -> &'static str {
        return OVH_NAME;
    }

    /// Sends an notification using the provided data.
    fn inventory(&self, all: bool) -> Result<Vec<ServerInfo>, LibError> {
        let response = self.query_available_servers(None)?;

        let response: Vec<OvhDedicatedServerInformation> = response
            .json()
            .map_err(|source| LibError::RequestError { source })?;

        let mut infos = Vec::new();

        for server in response.iter() {
            //skip unavailable except if requested
            if !server.is_available() && !all {
                continue;
            }

            infos.push(ServerInfo {
                reference: server.server.clone(),
                memory: server
                    .memory
                    .as_ref()
                    .unwrap_or(&"N/A".to_string())
                    .to_string(),
                storage: server
                    .storage
                    .as_ref()
                    .unwrap_or(&"N/A".to_string())
                    .to_string(),
                available: server.is_available(),
            });
        }

        Ok(infos)
    }

    /// Checks provider for the availability of a given server type.
    fn check(&self, server: &str) -> Result<bool, LibError> {
        let response = self.query_available_servers(Some(server))?;

        let response: Vec<OvhDedicatedServerInformation> = response
            .json()
            .map_err(|source| LibError::RequestError { source })?;

        if response.is_empty() {
            return Err(LibError::UnknownServer {
                server: server.to_string(),
            });
        }

        if response.len() > 1 {
            return Err(LibError::ApiError {
                message: format!("Multiple references found for server {}", server),
            });
        }

        Ok(response[0].is_available())
    }
}
