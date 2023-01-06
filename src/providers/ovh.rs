use std::env;

use colored::Colorize;
use reqwest::blocking::Response;
use serde::Deserialize;

use crate::LibError;

use super::{ProviderFactoryTrait, ProviderTrait, ServerInfo};

// OVH implementation

const ENV_NAME_OVH_EXCLUDE_DATACENTER: &str = "OVH_EXCLUDE_DATACENTER";

const OVH_URL: &str = "https://api.ovh.com/1.0/dedicated/server/datacenter/availabilities";

#[derive(Deserialize)]
struct OvhDedicatedServerInformation {
    datacenters: Vec<OvhDedicatedServerDatacenterAvailability>,
    memory: Option<String>,
    storage: Option<String>,
    server: String,
}

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

#[derive(Deserialize)]
struct OvhDedicatedServerDatacenterAvailability {
    datacenter: String,
    availability: String,
}

impl OvhDedicatedServerDatacenterAvailability {
    fn is_available(&self) -> bool {
        match self.availability.as_str() {
            "unavailable" => return false,
            "unknown" => return false,
            _ => return true,
        }
    }
}

pub struct Ovh {
    excluded_datacenters: Option<String>,
}

impl Ovh {
    fn new() -> Ovh {
        let p = env::var(ENV_NAME_OVH_EXCLUDE_DATACENTER)
            .unwrap_or_default()
            .trim()
            .to_string();
        Ovh {
            excluded_datacenters: if p.is_empty() { None } else { Some(p) },
        }
    }

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
    fn from_env() -> Result<Box<dyn ProviderTrait>, LibError> {
        Ok(Box::new(Ovh::new()))
    }
}

impl ProviderTrait for Ovh {
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