use axum::response::Html;
use axum::{routing::get, Router};
use color_eyre::eyre::Result;
use gcp_rust::instance::Instance;
use gcp_rust::operation::{self, *};
use gcp_rust::vpc::Vpc;
use serde::Deserialize;
use std::process::Command;
use std::str;
use std::sync::{Arc, Mutex};
use std::{thread, time};
use tokio::time::{sleep, Duration};
use url::Url;

const ENDPOINT_URL: &str = "https://www.googleapis.com/compute/v1";
#[derive(Debug)]
struct Client {
    project_id: String,
    token: Option<String>,
}
impl Client {
    pub fn new(project_id: &str) -> Self {
        Self {
            project_id: project_id.to_string(),
            token: None,
        }
    }
    pub fn get_token(&mut self) {
        let output = Command::new("gcloud")
            .args(["auth", "application-default", "print-access-token"])
            .output()
            .expect("failed to run gcloud command");
        self.token = Some(str::from_utf8(&output.stdout).unwrap().trim().to_string());
    }
    pub async fn create_instance(
        &self,
        instance: &Instance,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let zone = &instance.zone;
        let endpoint = format!("{ENDPOINT_URL}/projects/fred-hsu-veos/zones/{zone}/instances");
        let client = reqwest::Client::new();
        let token = self.token.as_ref().unwrap();
        client
            .post(endpoint)
            .bearer_auth(token)
            .json(&instance)
            .send()
            .await
    }
    pub async fn create_vpc(&self, vpc: &Vpc) -> Result<reqwest::Response, reqwest::Error> {
        let endpoint = format!("{ENDPOINT_URL}/projects/fred-hsu-veos/global/networks");
        let client = reqwest::Client::new();
        let token = self.token.as_ref().unwrap();
        client
            .post(endpoint)
            .bearer_auth(token)
            .json(&vpc)
            .send()
            .await
    }
    pub async fn delete_vpc(&self, go: &GlobalOperation) -> Result<GlobalOperation> {
        let endpoint = &go.target_link;
        println!("Deleting {endpoint}");
        let client = reqwest::Client::new();
        let token = self.token.as_ref().unwrap();
        let resp = client
            .delete(endpoint)
            .bearer_auth(token)
            .header(reqwest::header::CONTENT_LENGTH, 0)
            .send()
            .await?
            .json()
            .await?;
        Ok(resp)
    }
    pub async fn delete_instance(&self, zo: &ZonalOperation) -> Result<ZonalOperation> {
        let endpoint = &zo.target_link;
        println!("Deleting {endpoint}");
        let client = reqwest::Client::new();
        let token = self.token.as_ref().unwrap();
        Ok(client
            .delete(endpoint)
            .bearer_auth(token)
            .header(reqwest::header::CONTENT_LENGTH, 0)
            .send()
            .await?
            .json::<ZonalOperation>()
            .await?)
    }
    pub async fn wait_on_zonal_operation(&self, zo: &ZonalOperation) -> Result<ZonalOperation> {
        let endpoint = zo.zonal_wait_url();
        println!("Zonal wait url: {endpoint}");
        let client = reqwest::Client::new();
        let token = self.token.as_ref().unwrap();

        // Looping until we get a status of Done
        let z = loop {
            let resp = client
                .post(&endpoint)
                .bearer_auth(token)
                .header(reqwest::header::CONTENT_LENGTH, 0)
                .send()
                .await?
                .json::<ZonalOperation>()
                .await?;
            match resp.status {
                OperationStatus::Done => {
                    break resp;
                }
                _ => {
                    println!("Status is {:?}", resp.status);
                    continue;
                }
            }
        };
        Ok(z)
    }
    pub async fn wait_on_global_operation(&self, go: &GlobalOperation) -> Result<GlobalOperation> {
        let endpoint = go.wait_url();
        let client = reqwest::Client::new();
        let token = self.token.as_ref().unwrap();
        Ok(client
            .post(endpoint)
            .bearer_auth(token)
            .header(reqwest::header::CONTENT_LENGTH, 0)
            .send()
            .await?
            .json()
            .await?)
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "UPPERCASE")]
enum OperationStatus {
    Pending,
    Running,
    Done,
}

#[derive(Deserialize, Debug)]
enum Operations {
    Global(Operation),
    Regional(Operation),
    Zonal(Operation),
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Operation {
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

// TODO combine operations resources
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ZonalOperation {
    kind: String,
    id: String,
    name: String,
    zone: String,
    client_operation_id: Option<String>, // only if provided
    operation_type: String,
    target_link: String, // TODO use serde url feature
    target_id: String,   //uint64 format
    status: OperationStatus,
    user: String,
    progress: u32, //optional from 0-100
    insert_time: String,
    start_time: String,
    end_time: Option<String>,
    self_link: String, // TODO use serde url feature
}

impl ZonalOperation {
    fn get_project(&self) -> String {
        let url = Url::parse(&self.self_link).expect("cannot parse url");
        let segments = url
            .path_segments()
            .expect("cannot parse url")
            .collect::<Vec<&str>>();
        segments[3].to_string()
    }
    fn get_zone(&self) -> String {
        let url = Url::parse(&self.self_link).expect("cannot parse url");
        let segments = url
            .path_segments()
            .expect("cannot parse url")
            .collect::<Vec<&str>>();
        segments[5].to_string()
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
    pub fn zonal_wait_url(&self) -> String {
        let project = self.get_project();
        let resource_id = &self.id;
        let zone = &self.get_zone();
        format!("https://compute.googleapis.com/compute/v1/projects/{project}/zones/{zone}/operations/{resource_id}/wait");
        format!("{}/wait", &self.self_link)
    }
    pub fn zone_url(&self) -> String {
        let project = self.get_project();
        let resource_id = &self.id;
        let zone = &self.get_zone();
        format!("https://compute.googleapis.com/compute/v1/projects/{project}/zones/{zone}/operations/{resource_id}")
    }
}
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct GlobalOperation {
    kind: String,
    id: String,
    name: String,
    operation_type: String,
    target_link: String, // TODO use serde url feature
    progress: u32,
    insert_time: String,
    start_time: String,
    self_link: String, // TODO use serde url feature
}
impl GlobalOperation {
    fn get_project(&self) -> String {
        let url = Url::parse(&self.self_link).expect("cannot parse url");
        let segments = url
            .path_segments()
            .expect("cannot parse url")
            .collect::<Vec<&str>>();
        segments[3].to_string()
    }
    pub fn wait_url(&self) -> String {
        let project = self.get_project();
        let resource_id = &self.id;
        println!("wait_url: https://compute.googleapis.com/compute/v1/projects/{project}/global/operations/{resource_id}/wait");
        format!("https://compute.googleapis.com/compute/v1/projects/{project}/global/operations/{resource_id}/wait")
    }
}

async fn get_status(status: Arc<Mutex<bool>>) -> Html<&'static str> {
    if *status.lock().unwrap() {
        Html("done")
    } else {
        Html("in progress")
    }
}

async fn delete_all(
    instance_operation: &ZonalOperation,
    vpc_operation: &GlobalOperation,
    c: Client,
) {
    // delete vm
    let delete_op = c.delete_instance(instance_operation).await.unwrap();
    let resp = c.wait_on_zonal_operation(&delete_op).await.unwrap();
    println!("Response on wait for delete: \n{resp:?}");
    println!("finished deleting VM");

    // delete vpc
    let delete_vpc = c.delete_vpc(vpc_operation).await.unwrap();
    println!("finished deleting VPC");
}
async fn create_all(
    instance: &Instance,
    vpc: &Vpc,
    c: Client,
    is_ready: Arc<Mutex<bool>>,
) -> Result<()> {
    let vpc_operation = c.create_vpc(vpc).await?.json::<GlobalOperation>().await?;
    let resp = c.wait_on_global_operation(&vpc_operation).await?;
    println!("Waiting for vpc create to finish");
    let instance_operation = c
        .create_instance(instance)
        .await?
        .json::<ZonalOperation>()
        .await?;
    let resp = c.wait_on_zonal_operation(&instance_operation).await?;
    println!("Waiting for instance create to finish");
    {
        let mut r = is_ready.lock().unwrap();
        *r = true;
    }
    sleep(Duration::from_secs(10));
    delete_all(&instance_operation, &vpc_operation, c).await;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let vpcname = "fh-test-1";
    let instance_name = "fh-vm-1";
    let zone = "us-west1-a";
    let is_ready = Arc::new(Mutex::new(false));
    let mut c = Client::new("fred-hsu-veos");
    c.get_token();
    let vpc = Vpc::new(vpcname);
    let instance = Instance::new(instance_name, vpcname, zone);
    let create_ready = is_ready.clone();
    let app_ready = is_ready.clone();

    // Spawning a thread for creating resources so we can run the web server in parallel
    tokio::spawn(async move {
        create_all(&instance, &vpc, c, create_ready).await;
    });

    let app = Router::new().route("/", get(move || get_status(app_ready)));
    println!("Starting server");
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
    Ok(())
}
