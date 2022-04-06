use actix_cors::Cors;
use actix_web::middleware::{Compress, Logger};
use actix_web::{http::StatusCode, Error as ActixErr, HttpResponse};
use serde::{Deserialize, Serialize};
use sonic_channel::*;
use std::borrow::Borrow;
use std::sync::Mutex;

// fn main() -> result::Result<()> {
//     let channel = SearchChannel::start("localhost:21491", "SecretPassword")?;
//     let objects = channel.query("collection", "bucket", "recipe")?;
//     let suggestions = channel.suggest("collection", "bucket", "recipe")?;
//     dbg!(objects);
//     dbg!(suggestions);

//     Ok(())
// }

// fn main() -> result::Result<()> {
//     let channel = IngestChannel::start("localhost:21491", "SecretPassword")?;
//     let pushed = channel.push("collection", "bucket", "object:1", "my best recipe")?;
//     let pushed = channel.push("collection", "bucket", "object:1", "ankur")?;
//     let pushed = channel.push("collection", "bucket", "object:1", "ana")?;
//     let pushed = channel.push("collection", "bucket", "object:1", "mimi")?;
//     let pushed = channel.push("collection", "bucket", "object:1", "vikki")?;
//     // or
//     // let pushed = channel.push_with_locale("collection", "bucket", "object:1", "Мой лучший рецепт", "rus")?;
//     dbg!(pushed);

//     Ok(())
// }

// fn main() -> result::Result<()> {
//     let channel = ControlChannel::start("localhost:21491", "SecretPassword")?;
//     let result = channel.consolidate()?;
//     assert_eq!(result, true);

//     Ok(())
// }

use actix_web::{get, post, web, App, HttpServer, Responder};

#[derive(Serialize, Deserialize)]
struct Response {
    value: String,
}

struct Channels {
    pub ingest: Mutex<IngestChannel>,
    pub search: Mutex<SearchChannel>,
    pub control: Mutex<ControlChannel>,
}

#[get("/search/{name}")]
async fn search(
    name: web::Path<String>,
    channels: web::Data<Channels>,
) -> Result<HttpResponse, ActixErr> {
    let objects = channels
        .search
        .lock()
        .unwrap()
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
        .lock()
        .unwrap()
        .push("collection", "bucket", "object:1", &name)
        .unwrap();

    let resp = serde_json::json!({
        "status": published,
    });
    Ok(HttpResponse::build(StatusCode::from_u16(200).unwrap()).json(&resp))
}

#[post("/consolidate")]
async fn consolidate(channel: web::Data<Channels>) -> Result<HttpResponse, ActixErr> {
    let consolidated = channel.control.lock().unwrap().consolidate().unwrap();
    let resp = serde_json::json!({
        "consolidated": consolidated,
    });
    Ok(HttpResponse::build(StatusCode::from_u16(200).unwrap()).json(&resp))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // let objects = channel.query("collection", "bucket", "recipe")?;
    // let suggestions = channel.suggest("collection", "bucket", "recipe")?;
    HttpServer::new(move || {
        let search_channel = SearchChannel::start("localhost:21491", "SecretPassword").unwrap();
        let ingest_channel = IngestChannel::start("localhost:21491", "SecretPassword").unwrap();
        let control_channel = ControlChannel::start("localhost:21491", "SecretPassword").unwrap();

        let channels = Channels {
            search: Mutex::new(search_channel),
            ingest: Mutex::new(ingest_channel),
            control: Mutex::new(control_channel),
        };

        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET", "POST", "PUT"])
            .supports_credentials()
            .allow_any_header()
            .max_age(3600);
        App::new()
            .app_data(web::Data::new(channels))
            .wrap(cors)
            .service(search)
            .service(ingest)
            .service(consolidate)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
