//use color_eyre::eyre::Result;
use serde::Serialize;
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Vpc {
    auto_create_subnetworks: bool,
    name: String,
    routing_config: RoutingConfig,
}
impl Vpc {
    pub fn new(name: &str) -> Self {
        let routing_config = RoutingConfig {
            routing_mode: "REGIONAL".to_string(),
        };
        Self {
            auto_create_subnetworks: true,
            name: name.to_string(),
            routing_config,
        }
    }
}
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct RoutingConfig {
    routing_mode: String, // TODO: replace with enum?
}
