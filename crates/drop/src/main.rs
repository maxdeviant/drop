#[macro_use]
extern crate rocket;

mod database;
mod domain;

use std::path::Path;

use chrono::{DateTime, SecondsFormat, Utc};
use rocket::data::ToByteUnit;
use rocket::request::FromParam;
use rocket::serde::json::Json;
use rocket::serde::Serialize;
use rocket::Data;
use rocket_db_pools::{Connection, Database};
use tokio::fs::File;

use database::Db;
use domain::{
    entities::{ApiKeyId, DropId, UserId},
    ApiKeyValue,
};

impl<'a> FromParam<'a> for DropId {
    type Error = String;

    fn from_param(param: &'a str) -> Result<Self, Self::Error> {
        DropId::try_from(param)
            .map_err(|err| format!("failed to parse '{}' as a drop ID: {}", param, err))
    }
}

impl<'a> FromParam<'a> for UserId {
    type Error = String;

    fn from_param(param: &'a str) -> Result<Self, Self::Error> {
        UserId::try_from(param)
            .map_err(|err| format!("failed to parse '{}' as a user ID: {}", param, err))
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
    let filepath = Path::new(upload_dir).join(id.unprefixed());

    drop.open(2.mebibytes()).into_file(filepath).await?;

    Ok(id.to_string())
}

#[get("/drops/<id>")]
async fn get_drop(id: DropId) -> Option<File> {
    let upload_dir = "upload";
    let filepath = Path::new(upload_dir).join(id.unprefixed());

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
    let filepath = Path::new(upload_dir).join(id.unprefixed());

    let file = File::open(&filepath).await.unwrap();
    let file_metadata = File::metadata(&file).await.unwrap();

    Json(DropMetadata {
        created_at: id.created_at(),
        size_in_bytes: file_metadata.len(),
    })
}

#[post("/users")]
async fn create_user(mut db: Connection<Db>) -> database::Result<String> {
    let id = UserId::new();

    let unprefixed_id = id.unprefixed();
    let now = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);

    sqlx::query!(
        "insert into user (id, created_at, updated_at, username, full_name) values (?, ?, ?, ?, ?)",
        unprefixed_id,
        now,
        now,
        "maxdeviant",
        "Marshall Bowers"
    )
    .execute(&mut *db)
    .await?;

    Ok(id.to_string())
}

#[post("/users/<user_id>/keys")]
async fn generate_api_key(mut db: Connection<Db>, user_id: UserId) -> database::Result<String> {
    let id = ApiKeyId::new();
    let value = ApiKeyValue::new();

    let unprefixed_id = id.unprefixed();
    let unprefixed_user_id = user_id.unprefixed();
    let key_value = value.to_string();
    let now = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);

    sqlx::query!(
        "insert into api_key (id, created_at, updated_at, user_id, value) values (?, ?, ?, ?, ?)",
        unprefixed_id,
        now,
        now,
        unprefixed_user_id,
        key_value
    )
    .execute(&mut *db)
    .await?;

    Ok(id.to_string())
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let rocket = rocket::build()
        .mount(
            "/",
            routes![index, upload_drop, get_drop, get_drop_metadata,],
        )
        .mount("/x/", routes![create_user, generate_api_key])
        .attach(Db::init())
        .ignite()
        .await?;

    let _ = rocket.launch().await?;

    Ok(())
}
