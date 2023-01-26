use color_eyre::eyre::Result;
use serde::Deserialize;
use url::Url;

pub trait OperationMethod {
    fn delete() -> Result<()>;
    fn get() -> Result<Operation>;
    fn list() -> Result<Vec<Operation>>;
    fn wait() -> Result<Operation>;
}
#[derive(Deserialize, Debug)]
#[serde(rename_all = "UPPERCASE")]
enum Operations {
    Global(Operation),
    Regional(Operation),
    Zonal(Operation),
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "UPPERCASE")]
enum OperationStatus {
    Pending,
    Running,
    Done,
}

// TODO combine operations resources
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Operation {
    kind: String,
    id: String,
    name: String,
    zone: Option<String>,                // only for zonal
    region: Option<String>,              // only for regional
    client_operation_id: Option<String>, // only if provided
    operation_type: String,
    operation_group_id: Option<String>, // only for bulkInsert
    target_link: String,                // TODO use serde url feature
    target_id: String,                  //uint64 format
    status: OperationStatus,
    user: String,
    progress: u32, //optional from 0-100
    insert_time: String,
    start_time: String,
    status_message: Option<String>,
    end_time: Option<String>,    // only if completed
    self_link: String,           // TODO use serde url feature
    description: Option<String>, // only if provided
}

impl Operation {
    fn get_project(&self) -> String {
        let url = Url::parse(&self.self_link).expect("cannot parse url");
        let segments = url
            .path_segments()
            .expect("cannot parse url")
            .collect::<Vec<&str>>();
        segments[3].to_string()
    }
    fn get_region(&self) -> String {
        let url = Url::parse(&self.self_link).expect("cannot parse url");
        let segments = url
            .path_segments()
            .expect("cannot parse url")
            .collect::<Vec<&str>>();
        let zone = segments[5].to_string();
        zone.split('-').take(2).collect()
    }
    fn get_zone(&self) -> String {
        let url = Url::parse(&self.self_link).expect("cannot parse url");
        let segments = url
            .path_segments()
            .expect("cannot parse url")
            .collect::<Vec<&str>>();
        segments[5].to_string()
    }
    pub fn zonal_wait_url(&self) -> String {
        let project = self.get_project();
        let resource_id = &self.id;
        let zone = &self.get_zone();
        format!("https://compute.googleapis.com/compute/v1/projects/{project}/zones/{zone}/operations/{resource_id}/wait")
    }
    pub fn regional_wait_url(&self) -> String {
        let project = self.get_project();
        let resource_id = &self.id;
        let region = &self.get_region();
        format!("https://compute.googleapis.com/compute/v1/projects/{project}/regions/{region}/operations/{resource_id}/wait")
    }
    pub fn global_wait_url(&self) -> String {
        let project = self.get_project();
        let resource_id = &self.id;
        format!("https://compute.googleapis.com/compute/v1/projects/{project}/global/operations/{resource_id}/wait")
    }
}
