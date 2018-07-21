#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate dotenv;
extern crate harsh;
extern crate rocket;

use std::fs;
use std::fs::File;
use std::io;
use std::path::Path;

use dotenv::dotenv;
use harsh::HarshBuilder;
use rocket::http::{RawStr, Status};
use rocket::response::Response;
use rocket::Data;

#[get("/")]
fn index() -> io::Result<String> {
    let mut drops = Vec::new();
    for entry in fs::read_dir("upload")? {
        let entry = entry?;
        let id = entry.file_name().into_string().unwrap();
        drops.push(id);
    }

    let links = drops
        .iter()
        .map(|drop| {
            format!(
                "<a href=\"{host}/{id}\">{id}</a>",
                host = "http://localhost:8000",
                id = drop
            )
        })
        .collect::<Vec<String>>()
        .join("");
    let html = format!(
        "
        <!doctype html>
        <html>
        <head>
            <title>drop (滴)</title>
        </head>
        <body>
            <pre>drop (滴)</pre>

            {links}
        </body>
        </html>
    ",
        links = links
    );
    Ok(html)
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
        .launch();
}
