#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate chrono;
extern crate dotenv;
extern crate harsh;
extern crate rocket;
extern crate rocket_contrib;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::fs;
use std::fs::File;
use std::io;
use std::path::Path;

use chrono::{DateTime, Utc};
use dotenv::dotenv;
use harsh::HarshBuilder;
use rocket::http::{RawStr, Status};
use rocket::response::Response;
use rocket::Data;
use rocket_contrib::Template;

#[derive(Serialize)]
struct TemplateContext {
    base_url: String,
    drops: Vec<Drop>,
}

#[derive(Serialize, Deserialize)]
struct DropMetadata {
    created_on: DateTime<Utc>,
}

#[derive(Serialize)]
struct Drop {
    id: String,
    metadata: DropMetadata,
}

fn list_drops() -> io::Result<Vec<Drop>> {
    let mut drops = Vec::new();
    for entry in fs::read_dir("upload")? {
        let entry = entry?;
        let filename = entry.file_name().into_string().unwrap();
        if filename.ends_with(".meta") {
            continue;
        }
        let metadata_filename = format!(
            "{filename}.meta",
            filename = entry.path().into_os_string().into_string().unwrap()
        );
        let metadata_file = File::open(Path::new(&metadata_filename))?;
        let metadata: DropMetadata = serde_json::from_reader(metadata_file)?;

        drops.push(Drop {
            id: filename,
            metadata,
        });
    }
    Ok(drops)
}

#[get("/")]
fn index() -> io::Result<Template> {
    let drops = list_drops()?;

    let context = TemplateContext {
        base_url: String::from("http://localhost:8000"),
        drops,
    };

    Ok(Template::render("index", &context))
}

#[post("/", data = "<data>")]
fn upload(data: Data) -> io::Result<String> {
    let harsh = HarshBuilder::new().salt("滴").init().unwrap();
    let count = fs::read_dir("upload")?.count() as u64;
    let id = harsh.encode(&[count + 1]).unwrap();
    let filename = format!("upload/{id}", id = id);
    let url = format!("{host}/{id}\n", host = "http://localhost:8000", id = id);

    let metadata_filename = format!("{filename}.meta", filename = filename);
    let metadata = DropMetadata {
        created_on: Utc::now(),
    };
    let metadata_json = serde_json::to_string(&metadata)?;

    data.stream_to_file(Path::new(&filename))?;
    fs::write(&metadata_filename, metadata_json)?;

    Ok(url)
}

#[get("/<id>")]
fn retrieve(id: &RawStr) -> Response {
    let filename = format!("upload/{id}", id = id);
    let file = File::open(&filename).unwrap();
    Response::build()
        .status(Status::Ok)
        .streamed_body(file)
        .finalize()
}

fn main() {
    dotenv().ok();

    rocket::ignite()
        .mount("/", routes![index, upload, retrieve])
        .attach(Template::fairing())
        .launch();
}
