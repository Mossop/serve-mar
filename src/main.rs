/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::path::PathBuf;

use actix_files::NamedFile;
use actix_web::{get, web, App, HttpServer, Responder};
use clap::Parser;
use flexi_logger::Logger;

#[derive(Parser)]
#[clap(author, version, about)]
struct Args {
    pub mar_file: String,
}

struct AppData {
    mar_file: PathBuf,
}

#[get("/update.mar")]
async fn greet(data: web::Data<AppData>) -> impl Responder {
    NamedFile::open_async(&data.mar_file).await
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if let Err(e) = Logger::try_with_env_or_str("info").and_then(|logger| logger.start()) {
        eprintln!("Warning, failed to start logging: {}", e);
    }

    let args = Args::parse();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppData {
                mar_file: PathBuf::from(&args.mar_file),
            }))
            .service(greet)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
