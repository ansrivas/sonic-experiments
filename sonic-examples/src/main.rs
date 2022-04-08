use actix_cors::Cors;
use actix_files as fs;
use actix_web::middleware::{Compress, Logger};
use actix_web::{get, post, web, App, HttpServer, Responder};
use actix_web::{http::StatusCode, Error as ActixErr, HttpResponse};
use actix_web_static_files::ResourceFiles;
use askama_actix::TemplateToResponse;
use serde::{Deserialize, Serialize};
use sonic_channel::*;

mod assets;
mod template;

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

#[derive(Serialize, Deserialize)]
struct Response {
    value: String,
}

struct Channels {
    pub ingest: IngestChannel,
    pub search: SearchChannel,
    pub control: ControlChannel,
}

#[get("/search/{name}")]
async fn search(
    name: web::Path<String>,
    channels: web::Data<Channels>,
) -> Result<HttpResponse, ActixErr> {
    let objects = channels
        .search
        .suggest("collection", "bucket", &name)
        .unwrap();
    let resp: Vec<Response> = objects.into_iter().map(|o| Response { value: o }).collect();
    Ok(HttpResponse::build(StatusCode::from_u16(200).unwrap()).json(&resp))
}

#[post("/ingest/{name}")]
async fn ingest(
    name: web::Path<String>,
    channel: web::Data<Channels>,
) -> Result<HttpResponse, ActixErr> {
    let published = channel
        .ingest
        .push("collection", "bucket", "object:1", &name)
        .unwrap();

    let resp = serde_json::json!({
        "status": published,
    });
    Ok(HttpResponse::build(StatusCode::from_u16(200).unwrap()).json(&resp))
}

#[post("/consolidate")]
async fn consolidate(channel: web::Data<Channels>) -> Result<HttpResponse, ActixErr> {
    let consolidated = channel.control.consolidate().unwrap();
    let resp = serde_json::json!({
        "consolidated": consolidated,
    });
    Ok(HttpResponse::build(StatusCode::from_u16(200).unwrap()).json(&resp))
}

#[get("/")]
async fn index() -> Result<HttpResponse, ActixErr> {
    let resp = template::Search { name: "something" }.to_response();
    Ok(resp)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(move || {
        let search_channel = SearchChannel::start("localhost:21491", "SecretPassword").unwrap();
        let ingest_channel = IngestChannel::start("localhost:21491", "SecretPassword").unwrap();
        let control_channel = ControlChannel::start("localhost:21491", "SecretPassword").unwrap();

        let channels = Channels {
            search: search_channel,
            ingest: ingest_channel,
            control: control_channel,
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
            .app_data(web::Data::new(channels))
            .service(search)
            .service(ingest)
            .service(consolidate)
            .service(index)
            .service(ResourceFiles::new("/static", generate()))
        // .service(
        //     fs::Files::new("/static", ".")
        //         .show_files_listing()
        //         .use_last_modified(true),
        // )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
