#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate dotenv;
extern crate harsh;
extern crate rocket;
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;

use std::fs;
use std::fs::File;
use std::io;
use std::path::Path;

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

#[derive(Serialize)]
struct Drop {
    id: String,
}

#[get("/")]
fn index() -> io::Result<Template> {
    let mut filenames = Vec::new();
    for entry in fs::read_dir("upload")? {
        let entry = entry?;
        let id = entry.file_name().into_string().unwrap();
        filenames.push(id);
    }

    let drops = filenames
        .iter()
        .map(|filename| Drop {
            id: filename.clone(),
        })
        .collect::<Vec<Drop>>();

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

    data.stream_to_file(Path::new(&filename))?;
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
