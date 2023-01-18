use serde::{Deserialize, Serialize};
use std::process::Command;
use std::str;
use url::Url;

const ENDPOINT_URL: &str = "https://www.googleapis.com/compute/v1";
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Vpc {
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
#[derive(Debug)]
struct Client {
    project_id: String,
    token: Option<String>,
}
impl Client {
    pub fn new() -> Self {
        Self {
            project_id: "fred-hsu-veos".to_string(),
            token: None,
        }
    }
    pub fn get_token(&mut self) {
        let output = Command::new("gcloud")
            .args(["auth", "application-default", "print-access-token"])
            .output()
            .expect("failed to run gcloud command");
        self.token = Some(str::from_utf8(&output.stdout).unwrap().trim().to_string());
        println!("{:?}", self.token);
    }
    pub async fn create_vpc(&self, vpc: Vpc) -> Result<reqwest::Response, reqwest::Error> {
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
    pub async fn wait(&self) {
        let project = self.get_project();
        let resource_id = &self.id;
        let url = format!("https://compute.googleapis.com/compute/v1/projects/{project}/global/operations/{resource_id}/wait");
    }
}

/*
curl -X GET \
    -H "Authorization: Bearer $(gcloud auth application-default print-access-token)" \
    "https://cloudresourcemanager.googleapis.com/v3/projects/PROJECT_ID"
                */
#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let vpc = Vpc::new("fh-test-4");
    println!("{vpc:?}");

    let mut c = Client::new();
    c.get_token();
    let res = c.create_vpc(vpc).await?.json::<GlobalOperation>().await?;
    println!("{res:?}");
    println!("Resource ID: {}", res.id);
    println!("Project: {}", res.get_project());
    Ok(())
}
