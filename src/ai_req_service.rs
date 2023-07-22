use apalis::prelude::*;
use serde::{Deserialize, Serialize};
use skv::KeyValueStore;

use crate::http_client::APIRequestClient;

use std::error::Error;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AiReq {
    pub text: String,
}

impl Job for AiReq {
    const NAME: &'static str = "apalis::ai_req_service::AiReq";
}

pub async fn send_ai_req(
    job: AiReq,
    mut ctx: JobContext,
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let client = APIRequestClient::new();

    let kv_store = ctx.data::<KeyValueStore<String>>()?.clone();

    let text_prompt: &str = &job.text;
    let temperature = 0;
    let n_predict = 250;

    match client
        .send_request(text_prompt, temperature, n_predict)
        .await
    {
        Ok(response) => {
            println!("Response from the API:\n{}", response);
            match kv_store.insert(
                ctx.id().to_string(),
                // unescape the response
                response,
            ) {
                Ok(_) => {
                    ctx.set_status(JobState::Done);
                    println!("Job completed successfully");
                }
                _ => {
                    ctx.set_status(JobState::Failed);
                    println!("Job failed");
                }
            }
        }
        Err(err) => {
            eprintln!("Error occurred: {}", err);
            ctx.set_status(JobState::Failed);
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            return Err(Box::new(err));
        }
    }
    Ok(())
}
