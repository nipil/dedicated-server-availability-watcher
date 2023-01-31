use super::{ProviderFactoryTrait, ProviderTrait, ServerInfo};
use crate::LibError;
use http::{Method, StatusCode};
use reqwest::blocking::{Client, RequestBuilder, Response};
use serde::Deserialize;
use std::collections::HashMap;
use uuid::Uuid;

// Scaleway implementation

/// Common name to identify the provider
pub const SCALEWAY_NAME: &str = "scaleway";

/// Common environment variable to input your Scaleway API key.
const ENV_SCALEWAY_SECRET_KEY: &str = "SCALEWAY_SECRET_KEY";

/// Common environment variable to input your Scaleway API key.
const ENV_SCALEWAY_BAREMETAL_ZONES: &str = "SCALEWAY_BAREMETAL_ZONES";

/// Used for API result deserialisation, with only interesting fields implemented
#[derive(Deserialize)]
struct ScalewayBaremetalOffers {
    offers: Vec<ScalewayBaremetalOffer>,
}

#[derive(Deserialize, Clone)]
struct ScalewayBaremetalOfferMemory {
    capacity: u64,
}

#[derive(Deserialize, Clone)]
struct ScalewayBaremetalOfferDisk {
    capacity: u64,
}

#[derive(Deserialize, Clone)]
struct ScalewayBaremetalOffer {
    id: String,
    name: String,
    stock: String, // either "empty", "low", or "available"
    disks: Vec<ScalewayBaremetalOfferDisk>,
    enable: bool,
    memories: Vec<ScalewayBaremetalOfferMemory>,
}

/// Helps get availability
impl ScalewayBaremetalOffer {
    fn is_available(&self) -> bool {
        return self.enable && self.stock != "empty";
    }
}

// I prefer the From trait, as i can pass references
impl From<&ScalewayBaremetalOffer> for ServerInfo {
    /// Extracts only interesting information which is common to all providers
    fn from(offer: &ScalewayBaremetalOffer) -> Self {
        let memory = offer.memories.iter().map(|mem| mem.capacity).sum::<u64>() / 1000000000;
        let storage = offer.disks.iter().map(|disk| disk.capacity).sum::<u64>() / 1000000000;

        ServerInfo {
            reference: format!("{} ({})", offer.id, offer.name),
            memory: format!("{memory}G"),
            storage: format!("{storage}G"),
            available: offer.is_available(),
        }
    }
}

/// Gets server inventory and availability.
pub struct Scaleway {
    secret_key: String,
    zones: Vec<String>,
}

impl Scaleway {
    /// Builds a new instance.
    fn new(secret_key: &str, zones_csv: &str) -> Result<Self, LibError> {
        // Secret key is a UUID
        let secret_key = secret_key.to_string();
        Uuid::parse_str(&secret_key).map_err(|source| LibError::ValueError {
            name: "malformed scaleway secret key".to_string(),
            value: source.to_string(),
        })?;

        // split zones and verify that no zones is empty
        let zones: Vec<String> = zones_csv.split(',').map(|s| s.trim().to_string()).collect();
        if zones.iter().find(|i| i.is_empty()).is_some() {
            return Err(LibError::ValueError {
                name: "found empty scaleway zone".into(),
                value: zones_csv.into(),
            });
        }

        // construct the object if everything is ok
        Ok(Self { secret_key, zones })
    }

    /// Wrapper for automatic handling of authentication
    fn create_authenticated_request_builder(&self, method: Method, url: &str) -> RequestBuilder {
        Client::new()
            .request(method, url)
            .header("X-Auth-Token", &self.secret_key)
    }

    /// Fallback error handler for queries
    fn do_error_if_not_successful(response: &Response) -> Result<(), LibError> {
        if response.status().is_success() {
            return Ok(());
        }

        Err(LibError::ApiError {
            message: format!(
                "Error during Scaleway baremetal query: code {}",
                response.status()
            ),
        })
    }

    /// Executes simple authenticated get queries which fails only on transport errors
    fn get_api_authenticated(&self, url: &str) -> Result<Response, LibError> {
        let response = self
            .create_authenticated_request_builder(Method::GET, url)
            .send()
            .map_err(|source| LibError::RequestError { source })?;

        Ok(response)
    }

    /// Gets all offers in specified zone.
    fn get_zone_offers(&self, zone: &str) -> Result<ScalewayBaremetalOffers, LibError> {
        let url = format!("https://api.scaleway.com/baremetal/v1/zones/{zone}/offers");
        let response = self.get_api_authenticated(&url)?;

        // fallback error handler
        Self::do_error_if_not_successful(&response)?;

        // reqwest deserialize and check
        response
            .json::<ScalewayBaremetalOffers>()
            .map_err(|source| LibError::RequestError { source })
    }

    /// Inserts an offer into map if not already present, or override its availability if available
    fn insert_or_update_offer(
        map: &mut HashMap<String, ScalewayBaremetalOffer>,
        offer: &ScalewayBaremetalOffer,
    ) {
        map.entry(offer.id.clone())
            // update stored availability if current offer is "better"
            .and_modify(|info| {
                if offer.is_available() {
                    info.enable = offer.enable;
                    info.stock = offer.stock.clone();
                }
            })
            // insert offer if not already present (and only then does it copy)
            .or_insert(offer.clone());
    }

    /// Gets all offers.
    fn get_offers(&self) -> Result<Vec<ScalewayBaremetalOffer>, LibError> {
        let mut map: HashMap<String, ScalewayBaremetalOffer> = HashMap::new();

        for zone in &self.zones {
            // get all offers for specific zone
            let result = self.get_zone_offers(&zone)?;
            for offer in result.offers.iter() {
                // update offer availability across all zones
                Self::insert_or_update_offer(&mut map, offer);
            }
        }

        // Builds result by moving the values from the map into the vec
        Ok(Vec::from_iter(map.into_values()))
    }

    /// Gets a specific offer in specified zone
    fn get_zone_offer(
        &self,
        zone: &str,
        offer_id: &str,
    ) -> Result<Option<ScalewayBaremetalOffer>, LibError> {
        let url = format!("https://api.scaleway.com/baremetal/v1/zones/{zone}/offers/{offer_id}");
        let response = self.get_api_authenticated(&url)?;

        // the API returns 404 if 'offer_id' is not found, and we do not want to error out
        if response.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }

        // fallback error handler
        Self::do_error_if_not_successful(&response)?;

        // reqwest deserialize and check
        Ok(Some(
            response
                .json::<ScalewayBaremetalOffer>()
                .map_err(|source| LibError::RequestError { source })?,
        ))
    }

    /// Gets a specific offer.
    fn get_offer(&self, offer_id: &str) -> Result<ScalewayBaremetalOffer, LibError> {
        // Start with no result
        let mut result: Option<ScalewayBaremetalOffer> = None;

        for zone in &self.zones {
            match self.get_zone_offer(&zone, offer_id)? {
                // skip if we did not find an offer for this id
                None => continue,

                Some(offer) => {
                    // fill result if it was previously empty, so only the first makes an actual clone
                    let info = result.get_or_insert(offer.clone());
                    // if offer availability is 'better' than current value, update it
                    if !info.is_available() && offer.is_available() {
                        info.enable = offer.enable;
                        info.stock = offer.stock;
                    }
                }
            }
        }

        // We could have return an Option if on offer was found.
        // By choice, we chose to produce an error in that case.
        result.ok_or(LibError::UnknownServer {
            server: offer_id.to_string(),
        })
    }
}

impl ProviderFactoryTrait for Scaleway {
    /// Builds an Ovh provider from environment variables.
    fn from_env() -> Result<Box<dyn ProviderTrait>, LibError> {
        let secret_key = crate::get_env_var(ENV_SCALEWAY_SECRET_KEY)?;
        let zones_csv = crate::get_env_var(ENV_SCALEWAY_BAREMETAL_ZONES)?;
        Ok(Box::new(Self::new(&secret_key, &zones_csv)?))
    }
}

impl ProviderTrait for Scaleway {
    /// Gets the actual name of the provider.
    fn name(&self) -> &'static str {
        return SCALEWAY_NAME;
    }

    /// Collects provider inventory.
    fn inventory(&self, all: bool) -> Result<Vec<ServerInfo>, LibError> {
        Ok(self
            .get_offers()?
            .iter()
            .filter(|offer| offer.is_available() || all)
            .map(|offer| offer.into())
            .collect())
    }

    /// Checks provider for the availability of a given server type.
    fn check(&self, server: &str) -> Result<bool, LibError> {
        let offer = self.get_offer(server)?;
        Ok(offer.is_available())
    }
}
