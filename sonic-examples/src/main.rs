mod app_state;
mod assets;
mod db;
mod errors;
mod template;
mod utils;

use crate::db::{run_migrations, Postgres, Product};
use actix_cors::Cors;
use actix_web::middleware::{Compress, NormalizePath};
use actix_web::{get, post, web, App, HttpServer};
use actix_web::{http::StatusCode, Error as ActixErr, HttpResponse};
use actix_web_static_files::ResourceFiles;
use app_state::AppState;
use askama_actix::TemplateToResponse;
use errors::SonicErrors;
use serde::{Deserialize, Serialize};
use sonic_channel::*;

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

#[derive(Serialize, Deserialize, Debug)]
struct Response {
    value: String,
}

#[derive(Serialize, Deserialize)]
struct Request {
    text: String,
}

#[get("/search/{name}")]
async fn search(
    name: web::Path<String>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, ActixErr> {
    println!("{}", &name);

    state
        .search
        .ping()
        .map_err(|source| SonicErrors::Sonic { source })?;

    let words = name.split_ascii_whitespace();
    let mut indices = vec![];
    for word in words {
        let objects = state
            .search
            .query_with_limit("collection", "bucket", &word, 10)
            .map_err(|source| SonicErrors::Sonic { source })?;
        indices.extend_from_slice(&objects);
    }

    let resp: Vec<Response> = Postgres::query_products(&state.pgpool, indices.clone())
        .await?
        .iter()
        .map(|p| Response {
            value: p.details.clone(),
        })
        .collect();
    Ok(HttpResponse::build(StatusCode::from_u16(200).unwrap()).json(&resp))
}

#[post("/ingest")]
async fn ingest(
    text: web::Json<Request>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, ActixErr> {
    state
        .ingest
        .ping()
        .map_err(|source| SonicErrors::Sonic { source })?;

    let obj = uuid::Uuid::new_v4();

    Postgres::insert_product(
        &state.pgpool,
        &Product {
            details: text.text.clone(),
            object_id: obj,
            ..Default::default()
        },
    )
    .await?;

    let published = state
        .ingest
        .push("collection", "bucket", &obj.to_string(), &text.text)
        .map_err(|source| SonicErrors::Sonic { source })?;

    let resp = serde_json::json!({
        "status": published,
    });
    Ok(HttpResponse::build(StatusCode::from_u16(200).unwrap()).json(&resp))
}

#[post("/consolidate")]
async fn consolidate(state: web::Data<AppState>) -> Result<HttpResponse, ActixErr> {
    state
        .control
        .ping()
        .map_err(|source| SonicErrors::Sonic { source })?;

    let consolidated = state.control.consolidate().unwrap();
    let resp = serde_json::json!({
        "consolidated": consolidated,
    });
    Ok(HttpResponse::build(StatusCode::from_u16(200).unwrap()).json(&resp))
}

#[get("/")]
async fn index() -> Result<HttpResponse, ActixErr> {
    let resp = template::Search {}.to_response();
    Ok(resp)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pg_dsn = "postgres://testuser:testpassword@localhost/testdb";
    let pgpool = Postgres::setup(pg_dsn, 4, "sonic").await.unwrap();

    run_migrations(&pgpool).await.unwrap();

    HttpServer::new(move || {
        let search_channel = SearchChannel::start("localhost:21491", "SecretPassword").unwrap();
        let ingest_channel = IngestChannel::start("localhost:21491", "SecretPassword").unwrap();
        let control_channel = ControlChannel::start("localhost:21491", "SecretPassword").unwrap();

        let channels = AppState {
            search: search_channel,
            ingest: ingest_channel,
            control: control_channel,
            pgpool: pgpool.clone(),
        };

        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET", "POST", "PUT"])
            .supports_credentials()
            .allow_any_header()
            .max_age(3600);
        App::new()
            .wrap(cors)
            .wrap(Compress::default())
            .wrap(NormalizePath::trim())
            .app_data(web::Data::new(channels))
            .service(search)
            .service(ingest)
            .service(consolidate)
            .service(index)
            .service(ResourceFiles::new("/static", generate()))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
