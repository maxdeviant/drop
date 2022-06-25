#[macro_use]
extern crate rocket;

mod domain;

use std::path::Path;

use rocket::request::FromParam;
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

#[get("/drops/<id>")]
async fn get_drop(id: DropId) -> Option<File> {
    let upload_dir = "upload";
    let filename = Path::new(upload_dir).join(id.to_string());

    File::open(&filename).await.ok()
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let rocket = rocket::build()
        .mount("/", routes![index, get_drop])
        .ignite()
        .await?;

    let _ = rocket.launch().await?;

    Ok(())
}
