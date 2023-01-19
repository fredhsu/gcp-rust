use serde::Serialize;
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Instance {
    machine_type: String,
    name: String,
    disks: Vec<Disk>,
    network_interfaces: Vec<NetworkInterface>,
}
// TODO implement builder pattern for this
impl Instance {
    pub fn new(name: &str, subnetwork: &str, zone: &str) -> Self {
        let disks = vec![Disk::new(name)];
        //let subnetwork = "projects/fred-hsu-veos/regions/us-west1/subnetworks/fh-test-1";
        let network_interfaces = vec![NetworkInterface::new(subnetwork)];
        let machine_type = format!("projects/fred-hsu-veos/zones/{zone}/machineTypes/e2-micro");
        Self {
            disks,
            machine_type,
            name: name.to_string(),
            network_interfaces,
        }
    }
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Disk {
    auto_delete: bool,
    boot: bool,
    device_name: String,
    initialize_params: InitializeParam,
    mode: String,
    #[serde(rename = "type")]
    disk_type: String, // rename to type
}
impl Disk {
    pub fn new(device_name: &str) -> Self {
        Self {
            auto_delete: true,
            boot: true,
            device_name: device_name.to_string(),
            initialize_params: InitializeParam::new(),
            mode: "READ_WRITE".to_string(),
            disk_type: "PERSISTENT".to_string(),
        }
    }
}
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct InitializeParam {
    disk_size_gb: String,
    disk_type: String,
    //labels
    source_image: String,
}

impl InitializeParam {
    pub fn new() -> Self {
        Self {
            disk_size_gb: "10".to_string(),
            disk_type: "projects/fred-hsu-veos/zones/us-central1-a/diskTypes/pd-balanced"
                .to_string(),
            source_image: "projects/debian-cloud/global/images/debian-11-bullseye-v20221206"
                .to_string(),
        }
    }
}
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct NetworkInterface {
    access_configs: Vec<AccessConfig>,
    stack_type: String,
    subnetwork: String,
}
impl NetworkInterface {
    pub fn new(subnetwork: &str) -> Self {
        let access_configs = vec![AccessConfig {
            name: "External NAT".to_string(),
            network_tier: "PREMIUM".to_string(),
        }];
        Self {
            access_configs,
            stack_type: "IPV4_ONLY".to_string(),
            subnetwork: "projects/fred-hsu-veos/regions/us-west1/subnetworks/fh-test-1".to_string(),
        }
    }
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct AccessConfig {
    name: String,
    network_tier: String,
}
