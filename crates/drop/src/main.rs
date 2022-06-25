#[macro_use]
extern crate rocket;

mod domain;

use std::path::Path;

use chrono::{DateTime, Utc};
use rocket::data::ToByteUnit;
use rocket::request::FromParam;
use rocket::serde::json::Json;
use rocket::serde::Serialize;
use rocket::Data;
use tokio::fs::File;

use domain::DropId;

impl<'a> FromParam<'a> for DropId {
    type Error = String;

    fn from_param(param: &'a str) -> Result<Self, Self::Error> {
        DropId::try_from(param)
            .map_err(|err| format!("failed to parse '{}' as a drop ID: {}", param, err))
    }
}

#[get("/")]
async fn index() -> &'static str {
    "滴"
}

#[post("/drops", data = "<drop>")]
async fn upload_drop(drop: Data<'_>) -> std::io::Result<String> {
    let id = DropId::new();

    let upload_dir = "upload";
    let filepath = Path::new(upload_dir).join(id.to_string());

    drop.open(2.mebibytes()).into_file(filepath).await?;

    Ok(id.to_string())
}

#[get("/drops/<id>")]
async fn get_drop(id: DropId) -> Option<File> {
    let upload_dir = "upload";
    let filepath = Path::new(upload_dir).join(id.to_string());

    File::open(&filepath).await.ok()
}

#[derive(Debug, Serialize)]
pub struct DropMetadata {
    pub created_at: DateTime<Utc>,
    pub size_in_bytes: u64,
}

#[get("/drops/<id>/meta")]
async fn get_drop_metadata(id: DropId) -> Json<DropMetadata> {
    let upload_dir = "upload";
    let filepath = Path::new(upload_dir).join(id.to_string());

    let file = File::open(&filepath).await.unwrap();
    let file_metadata = File::metadata(&file).await.unwrap();

    Json(DropMetadata {
        created_at: id.created_at(),
        size_in_bytes: file_metadata.len(),
    })
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let rocket = rocket::build()
        .mount(
            "/",
            routes![index, upload_drop, get_drop, get_drop_metadata],
        )
        .ignite()
        .await?;

    let _ = rocket.launch().await?;

    Ok(())
}
