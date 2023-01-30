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
    datacenter: String,
}

impl OvhDedicatedServerDatacenterAvailability {
    /// Evaluates availability.
    fn is_available(&self) -> bool {
        match self.availability.as_str() {
            "unavailable" | "unknown" => return false,
            _ => return true,
        }
    }
}

// I prefer the Frommkdir  trait, as i can pass references
impl From<&OvhDedicatedServerInformation> for ServerInfo {
    /// Extracts only interesting information which is common to all providers
    fn from(info: &OvhDedicatedServerInformation) -> Self {
        ServerInfo {
            reference: format!(
                "{} (@{})",
                info.server,
                info.datacenters
                    .iter()
                    .map(|d| d.datacenter.clone())
                    .collect::<Vec<String>>()
                    .join(",")
            ),
            memory: info
                .memory
                .as_ref()
                .unwrap_or(&"N/A".to_string())
                .to_string(),
            storage: info
                .storage
                .as_ref()
                .unwrap_or(&"N/A".to_string())
                .to_string(),
            available: info.is_available(),
        }
    }
}

/// Gets server inventory and availability.
pub struct Ovh {
    /// Used to exclude datacenters by their id.
    /// Examples : ["ca","bhs","fr","gra","rbx","sbg"]
    excluded_datacenters: Vec<String>,
}

impl Ovh {
    /// Builds a new instance.
    fn new(excluded_datacenters: &Option<String>) -> Result<Self, LibError> {
        let excluded_datacenters = crate::tokenize_optional_csv_str(&excluded_datacenters)?;
        Ok(Self {
            excluded_datacenters,
        })
    }

    /// Gets availability for specified server types.
    /// `server`: optionally used to query for a single server type.
    fn api_get_dedicated_server_datacenter_availabilities(
        &self,
        server: Option<&str>,
    ) -> Result<Vec<OvhDedicatedServerInformation>, LibError> {
        let mut query: Vec<(&str, String)> = Vec::new();

        // Handle optional datacenter exclusions.
        if self.excluded_datacenters.is_empty() {
            query.push(("excludeDatacenters", "false".into()));
        } else {
            query.push(("excludeDatacenters", "true".into()));
            query.push(("datacenters", self.excluded_datacenters.join(",")));
        }

        // Handles optional server filtering.
        if let Some(server) = server {
            query.push(("server", server.into()));
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

        let results: Vec<OvhDedicatedServerInformation> = response
            .json()
            .map_err(|source| LibError::RequestError { source })?;

        Ok(results)
    }
}

impl ProviderFactoryTrait for Ovh {
    /// Builds an Ovh provider from environment variables.
    fn from_env() -> Result<Box<dyn ProviderTrait>, LibError> {
        let excluded_datacenters = crate::get_env_var_option(ENV_NAME_OVH_EXCLUDE_DATACENTER);
        Ok(Box::new(Ovh::new(&excluded_datacenters)?))
    }
}

impl ProviderTrait for Ovh {
    /// Gets the actual name of the provider.
    fn name(&self) -> &'static str {
        return OVH_NAME;
    }

    /// Collects provider inventory.
    fn inventory(&self, all: bool) -> Result<Vec<ServerInfo>, LibError> {
        let results = self.api_get_dedicated_server_datacenter_availabilities(None)?;

        let mut infos = Vec::new();

        for server in results.iter() {
            //skip unavailable except if requested
            if !server.is_available() && !all {
                continue;
            }

            infos.push(server.into());
        }

        Ok(infos)
    }

    /// Checks provider for the availability of a given server type.
    fn check(&self, server: &str) -> Result<bool, LibError> {
        let mut results = self.api_get_dedicated_server_datacenter_availabilities(Some(server))?;

        match results.pop() {
            None => Err(LibError::UnknownServer {
                server: server.to_string(),
            }),
            Some(result) => {
                results
                    .is_empty()
                    .then_some(result.is_available())
                    .ok_or(LibError::ApiError {
                        message: format!("Multiple references found for server {server}"),
                    })
            }
        }
    }
}
