use std::path::Path;

use tokio::fs::File;

#[macro_use]
extern crate rocket;

#[get("/")]
async fn index() -> &'static str {
    "滴"
}

#[get("/drops/<id>")]
async fn get_drop(id: String) -> Option<File> {
    let upload_dir = "upload";
    let filename = Path::new(upload_dir).join(id);

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
