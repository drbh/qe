mod ai_req_service;
mod http_client;

use actix_web::{web, App, HttpResponse, HttpServer};
use ai_req_service::{send_ai_req, AiReq};
use anyhow::Result;
use apalis::{
    layers::{DefaultRetryPolicy, Extension, RetryLayer, TraceLayer},
    prelude::*,
    sqlite::SqliteStorage,
};
use chrono::Utc;
use skv::KeyValueStore;
use sqlx::SqlitePool;
use tower::ServiceBuilder;

async fn produce_ai_reqs(storage: &SqliteStorage<AiReq>) -> Result<()> {
    let mut storage = storage.clone();
    for i in 0..1 {
        storage
            .schedule(
                AiReq {
                    text: "Write a haiku about cheese.".to_string(),
                },
                Utc::now() + chrono::Duration::seconds(i),
            )
            .await?;
    }
    Ok(())
}

async fn push_ai_req(
    ai_req: web::Json<AiReq>,
    storage: web::Data<(SqliteStorage<AiReq>, KeyValueStore<String>)>,
) -> HttpResponse {
    let (storage, _kv) = &*storage.into_inner();
    let mut storage = storage.clone();
    let res = storage.push(ai_req.into_inner()).await;
    match res {
        Ok(jid) => HttpResponse::Ok().body(format!("Request with job_id [{jid}] added to queue")),
        Err(e) => HttpResponse::InternalServerError().body(format!("{e}")),
    }
}

// read from kv based on job_id
async fn get_ai_req(
    job_id: web::Path<String>,
    storage: web::Data<(SqliteStorage<AiReq>, KeyValueStore<String>)>,
) -> HttpResponse {
    println!("job_id: {:?}", &job_id);

    let (_storage, kv_store): &(SqliteStorage<AiReq>, KeyValueStore<String>) =
        &storage.into_inner();
    match kv_store.get(&job_id.clone()) {
        Ok(res) => match res {
            Some(res) => HttpResponse::Ok().body(res),
            None => HttpResponse::Ok().body(format!("No result found for job_id: {:?}", &job_id)),
        },
        Err(e) => HttpResponse::InternalServerError().body(format!("{e}")),
    }
}

#[actix_web::main]
async fn main() -> Result<()> {
    // env_logger::init();

    std::env::set_var("RUST_LOG", "debug,sqlx::query=error");
    tracing_subscriber::fmt::init();

    let pool = SqlitePool::connect("sqlite::memory:").await?;

    let ai_req_storage: SqliteStorage<AiReq> = SqliteStorage::new(pool.clone());

    ai_req_storage
        .setup()
        .await
        .expect("unable to run migrations for sqlite");

    produce_ai_reqs(&ai_req_storage).await?;

    let (_stop_tx, _stop_rx) = tokio::sync::oneshot::channel::<()>();

    let kv_store: KeyValueStore<String> =
        if let Ok(kv_store) = KeyValueStore::load("kv_store.db", "kv_index.db") {
            kv_store
        } else {
            KeyValueStore::new("kv_store.db", "kv_index.db").unwrap()
        };

    let data = web::Data::new((ai_req_storage.clone(), kv_store.clone()));
    let http = async {
        HttpServer::new(move || {
            App::new()
                .app_data(data.clone())
                .service(web::resource("/get/{job_id}").route(web::get().to(get_ai_req)))
                .service(web::scope("/ai").route("/push", web::post().to(push_ai_req)))
        })
        .bind("127.0.0.1:8000")?
        .run()
        .await
    };

    let service = ServiceBuilder::new()
        .layer(RetryLayer::new(DefaultRetryPolicy))
        .service(job_fn(send_ai_req));

    let worker = Monitor::new()
        .register(
            WorkerBuilder::new("tasty-banana")
                .layer(TraceLayer::new())
                .layer(Extension(kv_store))
                .with_storage(ai_req_storage.clone())
                .build(service),
        )
        .run_with_signal(tokio::signal::ctrl_c());

    tokio::select! {
        res = http => res?,
        res = worker => res?,
    }

    Ok(())
}
