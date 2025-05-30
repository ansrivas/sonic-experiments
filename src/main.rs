mod app_state;
mod assets;
mod channel;
mod db;
mod errors;
mod template;
mod utils;

use askama::Template;
use itertools::Itertools;

use crate::channel::Channel;
use crate::db::{Postgres, Product, run_migrations};
use actix_cors::Cors;
use actix_web::middleware::{Compress, NormalizePath};
use actix_web::web::Html;
use actix_web::{App, HttpServer, get, post, web};
use actix_web::{Error as ActixErr, HttpResponse, http::StatusCode};
use actix_web_static_files::ResourceFiles;
use app_state::AppState;
use askama_actix::TemplateToResponse;
use errors::SonicErrors;
use serde::{Deserialize, Serialize};
use sonic_channel::*;
use std::collections::HashMap;

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

#[derive(Serialize, Deserialize, Debug)]
struct Response {
    value: String,
}

#[derive(Serialize, Deserialize)]
struct Request {
    text: String,
}

// impl Responder for AppError {
//     type Body = String;

//     fn respond_to(self, req: &HttpRequest) -> HttpResponse<Self::Body> {
//         // The error handler uses an askama template to display its content.
//         // The member `lang` is used by "_layout.html" which "error.html" extends. Even though it
//         // is always the fallback language English in here, "_layout.html" expects to be able to
//         // access this field, so you have to provide it.
//         #[derive(Debug, Template)]
//         #[template(path = "error.html")]
//         struct Tmpl<'a> {
//             req: &'a HttpRequest,
//             lang: Lang,
//             err: &'a AppError,
//         }

//         let tmpl = Tmpl {
//             req,
//             lang: Lang::default(),
//             err: &self,
//         };
//         if let Ok(body) = tmpl.render() {
//             (Html::new(body), self.status_code()).respond_to(req)
//         } else {
//             ("Something went wrong".to_string(), self.status_code()).respond_to(req)
//         }
//     }
// }

#[get("/search/{name}")]
async fn search(
    name: web::Path<String>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, ActixErr> {
    let search = state.channel.search();

    let mut indices = vec![];
    for word in name.split_ascii_whitespace() {
        let objects = search
            .query(QueryRequest::new(
                Dest::col_buc("collection", "bucket"),
                word,
            ))
            .map_err(|source| SonicErrors::Sonic { source })?;
        indices.extend_from_slice(&objects);
    }

    let unique_indices: Vec<String> = indices.into_iter().unique().collect();

    let products = Postgres::query_products(&state.pgpool, unique_indices.clone()).await?;
    let cache: HashMap<uuid::Uuid, Product> =
        products.into_iter().map(|p| (p.object_id, p)).collect();

    let resp: Vec<Response> = unique_indices
        .iter()
        .map(|i| {
            let u = uuid::Uuid::parse_str(i).unwrap();
            cache.get(&u).unwrap()
        })
        .collect::<Vec<&Product>>()
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
    let ingest = state.channel.ingest();

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
    let dest = Dest::col_buc("collection", "bucket").obj(&obj.to_string());
    let published = ingest
        .push(PushRequest::new(dest, &text.text))
        .map_err(|source| SonicErrors::Sonic { source })?;

    let resp = serde_json::json!({
        "status": published,
    });
    Ok(HttpResponse::build(StatusCode::from_u16(200).unwrap()).json(&resp))
}

#[post("/consolidate")]
async fn consolidate(state: web::Data<AppState>) -> Result<HttpResponse, ActixErr> {
    let control = state.channel.control();
    let consolidated = control.consolidate().unwrap();
    let resp = serde_json::json!({
        "consolidated": consolidated,
    });
    Ok(HttpResponse::build(StatusCode::from_u16(200).unwrap()).json(&resp))
}

#[get("/")]
async fn index() -> Result<impl actix_web::Responder, ActixErr> {
    Ok(Html::new(
        template::Search {}
            .render()
            .map_err(|e| SonicErrors::Render(e))?,
    ))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pg_dsn = "postgres://testuser:testpassword@localhost/testdb";
    let pgpool = Postgres::setup(pg_dsn, 4, "sonic").await.unwrap();

    run_migrations(&pgpool).await.unwrap();

    HttpServer::new(move || {
        let channels = AppState {
            channel: Channel::new("localhost:21491", "", "SecretPassword"),
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
