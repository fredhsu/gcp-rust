use color_eyre::eyre::Result;
use gcp_rust::instance::Instance;
use gcp_rust::vpc::Vpc;
use serde::Deserialize;
use std::process::Command;
use std::str;
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
        println!("{:?}", self.token);
    }
    pub async fn create_instance(
        &self,
        instance: Instance,
        zone: &str,
    ) -> Result<reqwest::Response, reqwest::Error> {
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
    pub async fn wait_on_global_operation(&self, go: GlobalOperation) -> Result<GlobalOperation> {
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

/*
curl -X GET \
    -H "Authorization: Bearer $(gcloud auth application-default print-access-token)" \
    "https://cloudresourcemanager.googleapis.com/v3/projects/PROJECT_ID"
                */
#[tokio::main]
async fn main() -> Result<()> {
    let vpcname = "fh-test-3";
    let zone = "us-west1-a";
    let vpc = Vpc::new(vpcname);

    let mut c = Client::new("fred-hsu-veos");
    c.get_token();
    let go = c.create_vpc(vpc).await?.json::<GlobalOperation>().await?;
    println!("Waiting for vpc create to finish");
    let resp = c.wait_on_global_operation(go).await?;
    println!("Response on wait: {resp:?}");
    let instance = Instance::new("fh-vm-1", vpcname, zone);
    //let ci = c.create_instance(instance, zone).await?.json::<RegionalOperation>().await?;
    let ci = c.create_instance(instance, zone).await?.text().await?;
    println!("ci: {ci:?}");
    Ok(())
}
