use super::{ProviderFactoryTrait, ProviderTrait, ServerInfo};
use crate::LibError;
use array_tool::vec::Intersect;
use http::Method;
use reqwest::blocking::{Client, RequestBuilder, Response};
use serde::Deserialize;
use serde_json::Value;

// Online implementation

/// Common name to identify the provider
pub const ONLINE_NAME: &str = "online";

/// Common environment variable to input your Online API key.
const ENV_ONLINE_PRIVATE_TOKEN: &str = "ONLINE_PRIVATE_TOKEN";

/// Common environment variable to input your Online API key.
const ENV_ONLINE_DATACENTERS: &str = "ONLINE_DATACENTERS";

/// Used for API result deserialisation, with only interesting fields implemented
#[derive(Deserialize)]
struct OnlineDediboxProduct {
    id: u32,
    slug: String,
    specs: OnlineDediboxProductSpecs,
    stocks: Vec<OnlineDediboxProductStock>,
}

impl OnlineDediboxProduct {
    /// Convenience function to detemine availability
    fn is_available(&self) -> bool {
        for stock in self.stocks.iter() {
            if stock.stock > 0 {
                return true;
            }
        }
        return false;
    }
}

/// Used for API result deserialisation, with only interesting fields implemented
#[derive(Deserialize)]
struct OnlineDediboxProductSpecs {
    cpu: String,
    ram: String,
    disks: String,
}

/// Used for API result deserialisation, with only interesting fields implemented
#[derive(Deserialize)]
struct OnlineDediboxProductStock {
    datacenter: OnlineDediboxProductDatacenter,
    stock: u32,
}

/// Used for API result deserialisation, with only interesting fields implemented
#[derive(Deserialize)]
struct OnlineDediboxProductDatacenter {
    name: String,
}

/// Used for API result deserialisation, with only interesting fields implemented
#[derive(Deserialize)]
struct OnlineDediboxProductAvailability {
    available: bool,
    datacenters: Vec<OnlineDediboxProductDatacenter>,
}

// I prefer the From trait, as i can pass references
impl From<&OnlineDediboxProduct> for ServerInfo {
    /// Extracts only interesting information which is common to all providers
    fn from(product: &OnlineDediboxProduct) -> Self {
        let mut cpu = product.specs.cpu.clone();
        cpu.retain(|c| !c.is_whitespace());

        let mut memory = product.specs.ram.clone();
        memory.retain(|c| !c.is_whitespace());

        let mut storage = product.specs.disks.clone();
        storage.retain(|c| !c.is_whitespace());

        let datacenters = product
            .stocks
            .iter()
            .map(|p| p.datacenter.name.clone())
            .collect::<Vec<String>>()
            .join(",");

        let available_quantity = product.stocks.iter().map(|p| p.stock).sum::<u32>();

        let reference = format!("{} ({}@{})", product.id, product.slug, datacenters);

        ServerInfo {
            reference,
            memory,
            storage,
            available: available_quantity > 0,
        }
    }
}

/// Gets server inventory and availability.
pub struct Online {
    api_token: String,
    datacenters: Vec<String>,
}

impl Online {
    /// Builds a new instance.
    fn new(api_token: &str, dc_csv: &Option<String>) -> Result<Self, LibError> {
        let api_token = api_token.to_string();
        if api_token.is_empty() {
            return Err(LibError::ValueError {
                name: "found empty online api token".into(),
                value: api_token.into(),
            });
        }

        // verify datacenter variable
        let datacenters: Vec<String> = crate::tokenize_optional_csv_str(dc_csv)?;

        // construct the object if everything is ok
        Ok(Self {
            api_token,
            datacenters,
        })
    }

    /// Wrapper for automatic handling of authentication
    fn create_authenticated_request_builder(&self, method: Method, url: &str) -> RequestBuilder {
        Client::new()
            .request(method, url)
            .header("Authorization", format!("Bearer {}", &self.api_token))
    }

    /// Fallback error handler for queries
    fn do_error_if_not_successful(response: &Response) -> Result<(), LibError> {
        if response.status().is_success() {
            return Ok(());
        }

        Err(LibError::ApiError {
            message: format!(
                "Error during Online dedibox query: code {}",
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

    // Extract the enum value from a serde_json Value::Object variant
    fn extract_serde_value_object_variant_value(
        name: &str,
        value: serde_json::Value,
    ) -> Result<serde_json::Map<String, Value>, LibError> {
        match value {
            Value::Object(value) => Ok(value),
            _ => Err(LibError::ApiError {
                message: format!("Dedibox value {name} is not a json object"),
            }),
        }
    }

    /// Gets all plans, with produc ranges and actual products
    fn get_plans(&self) -> Result<Vec<OnlineDediboxProduct>, LibError> {
        let url = "https://api.online.net/api/v1/dedibox/plans";
        let response = self.get_api_authenticated(&url)?;

        // fallback error handler
        Self::do_error_if_not_successful(&response)?;

        // reqwest generic deserialize
        let ranges = response
            .json::<Value>()
            .map_err(|source| LibError::RequestError { source })?;

        // extract enum value
        let ranges = Self::extract_serde_value_object_variant_value("root", ranges)?;

        let mut results: Vec<OnlineDediboxProduct> = Vec::new();
        for (range_name, products) in ranges.into_iter() {
            // convert range Value into its map
            let products = Self::extract_serde_value_object_variant_value(&range_name, products)?;

            for (_, product) in products.into_iter() {
                // deserialize product Value
                let product: OnlineDediboxProduct = serde_json::from_value(product)
                    .map_err(|source| LibError::JsonError { source })?;

                // add to collection
                results.push(product);
            }
        }

        Ok(results)
    }

    /// Gets a specific dedicated server product availability
    fn get_product_availability(&self, product_id: &str) -> Result<bool, LibError> {
        let url = format!("https://api.online.net/api/v1/dedibox/availability/{product_id}");
        let response = self.get_api_authenticated(&url)?;

        // fallback error handler
        Self::do_error_if_not_successful(&response)?;

        // reqwest deserialize and check
        let result = response
            .json::<OnlineDediboxProductAvailability>()
            .map_err(|source| LibError::RequestError { source })?;

        // if we do not filter on datacenters, any of them will be fine
        if self.datacenters.len() == 0 {
            return Ok(result.available);
        }

        // extract available datacenter names, and find if any are in common with desired ones
        let result: Vec<String> = result.datacenters.iter().map(|d| d.name.clone()).collect();
        Ok(self.datacenters.intersect(result).len() > 0)
    }
}

impl ProviderFactoryTrait for Online {
    /// Builds an Ovh provider from environment variables.
    fn from_env() -> Result<Box<dyn ProviderTrait>, LibError> {
        let api_token = crate::get_env_var(ENV_ONLINE_PRIVATE_TOKEN)?;
        let dc_csv = crate::get_env_var_option(ENV_ONLINE_DATACENTERS);
        Ok(Box::new(Self::new(&api_token, &dc_csv)?))
    }
}

impl ProviderTrait for Online {
    /// Gets the actual name of the provider.
    fn name(&self) -> &'static str {
        return ONLINE_NAME;
    }

    /// Collects provider inventory.
    fn inventory(&self, all: bool) -> Result<Vec<ServerInfo>, LibError> {
        Ok(self
            .get_plans()?
            .iter()
            .filter(|product| product.is_available() || all)
            .map(|offer| offer.into())
            .collect())
    }

    /// Checks provider for the availability of a given server type.
    fn check(&self, server: &str) -> Result<bool, LibError> {
        self.get_product_availability(server)
    }
}
