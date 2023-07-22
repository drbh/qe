use reqwest::{Client, Error};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenResponse {
    pub candidates: Vec<Candidate>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Candidate {
    pub output: String,
    pub safety_ratings: Vec<SafetyRating>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SafetyRating {
    pub category: String,
    pub probability: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct RequestPayload {
    prompt: String,
    temperature: usize,
    n_predict: usize,
}

#[derive(Debug)]
pub struct APIRequestClient {
    api_url: String,
    client: Client,
}

const API_URL: &str = "http://localhost:8080";

impl APIRequestClient {
    pub fn new() -> Self {
        let client = Client::new();
        Self {
            api_url: API_URL.to_string(),
            client,
        }
    }

    async fn post_data<T: Serialize>(&self, endpoint: &str, data: &T) -> Result<String, Error> {
        let body = json!(data).to_string();
        let url = format!("{}/{}", self.api_url, endpoint);

        println!("=================== Request ===================");
        println!("body: {}", body);
        println!("url: {}", url);
        println!("=================== Request ===================");

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await?;

        let response_text = response.text().await?;
        Ok(response_text)
    }

    pub async fn send_request(
        &self,
        text: &str,
        temperature: usize,
        n_predict: usize,
    ) -> Result<String, Error> {
        let payload = RequestPayload {
            prompt: text.to_string(),
            temperature,
            n_predict,
        };

        self.post_data("completion", &payload).await
    }
}
