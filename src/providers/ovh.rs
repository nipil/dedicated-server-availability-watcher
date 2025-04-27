use super::{ProviderFactoryTrait, ProviderTrait, ServerInfo};
use crate::{api_error_check, reqwest_blocking_builder_send, LibError};
use reqwest::blocking::Client;
use serde::Deserialize;
use tracing::{debug, trace};

// OVH implementation

/// Common name to identify the provider
pub const OVH_NAME: &str = "ovh";

/// Common environment variable to eventually filter the queries.
const ENV_NAME_OVH_EXCLUDE_DATACENTER: &str = "OVH_EXCLUDE_DATACENTER";

/// Provider API endpoint.
const OVH_URL: &str = "https://api.ovh.com/1.0/dedicated/server/datacenter/availabilities";

/// Used for API result deserialisation, with only interesting fields implemented
#[derive(Deserialize, Debug)]
struct OvhDedicatedServerInformation {
    datacenters: Vec<OvhDedicatedServerDatacenterAvailability>,
    memory: Option<String>,
    storage: Option<String>,
    server: String,
}

impl OvhDedicatedServerInformation {
    /// Convenience function to determine availability
    fn is_available(&self) -> bool {
        for datacenter in self.datacenters.iter() {
            if datacenter.is_available() {
                return true;
            }
        }
        false
    }
}

/// Used for API result deserialisation, with only interesting fields implemented
#[derive(Deserialize, Debug)]
struct OvhDedicatedServerDatacenterAvailability {
    availability: String,
    datacenter: String,
}

impl OvhDedicatedServerDatacenterAvailability {
    /// Convenience function to determine availability
    fn is_available(&self) -> bool {
        match self.availability.as_str() {
            "unavailable" | "unknown" => false,
            _ => true,
        }
    }
}

// I prefer the From trait, as I can pass references
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

        // Actual request
        let builder = Client::new().get(OVH_URL).query(&query);
        let response = reqwest_blocking_builder_send(builder)
            .map_err(|source| LibError::RequestError { source })?;
        let response = api_error_check(response, "OVH request error")?;

        // Deserialization
        let results: Vec<OvhDedicatedServerInformation> = response
            .json()
            .map_err(|source| LibError::RequestError { source })?;

        trace!("OVH response: {results:?}");
        Ok(results)
    }

    /// A filtered collection of ServerInfo from raw Ovh server information
    fn get_servers_info(
        &self,
        server: Option<&str>,
        include_unavailable: bool,
    ) -> Result<Vec<ServerInfo>, LibError> {
        let servers = self
            .api_get_dedicated_server_datacenter_availabilities(server)?
            .iter()
            .map(|item| ServerInfo::from(item))
            .filter(|item: &ServerInfo| item.available || include_unavailable)
            .collect();
        debug!("Servers info : {servers:?}");
        Ok(servers)
    }
}

impl ProviderFactoryTrait for Ovh {
    /// Builds an Ovh provider from environment variables.
    fn from_env() -> Result<Box<dyn ProviderTrait>, LibError> {
        let excluded_datacenters = crate::get_env_var_option(ENV_NAME_OVH_EXCLUDE_DATACENTER);
        Ok(Box::new(Self::new(&excluded_datacenters)?))
    }
}

impl ProviderTrait for Ovh {
    /// Gets the actual name of the provider.
    fn name(&self) -> &'static str {
        OVH_NAME
    }

    /// Collects provider inventory.
    fn inventory(&self, include_unavailable: bool) -> Result<Vec<ServerInfo>, LibError> {
        self.get_servers_info(None, include_unavailable)
    }

    /// Checks provider for the availability of a given server type.
    fn check(&self, server: &str) -> Result<bool, LibError> {
        self.get_servers_info(Some(server), false)
            .map(|servers| servers.len() > 0)
    }
}
