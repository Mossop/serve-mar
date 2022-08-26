/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::path::PathBuf;

use actix_files::NamedFile;
use actix_web::{get, web, App, HttpServer, Responder};
use clap::Parser;
use flexi_logger::Logger;
use update::Updates;

mod update;

#[derive(Parser)]
#[clap(author, version, about)]
struct Args {
    /// The mar file to serve.
    pub mar_file: String,
}

struct AppData {
    mar_file: PathBuf,
    updates: Updates,
}

#[get("/update.xml")]
async fn update_xml(data: web::Data<AppData>) -> impl Responder {
    log::info!("Request: /update.xml");
    data.updates.serialize()
}

#[get("/update.mar")]
async fn update_mar(data: web::Data<AppData>) -> impl Responder {
    log::info!("Request: /update.mar");
    NamedFile::open_async(&data.mar_file).await
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if let Err(e) =
        Logger::try_with_env_or_str("info,actix_server=warn").and_then(|logger| logger.start())
    {
        eprintln!("Warning, failed to start logging: {}", e);
    }

    let args = Args::parse();
    let updates = Updates::from_mar(&args.mar_file)?;

    log::info!("Service updates from http://localhost:8000/update.xml");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppData {
                mar_file: PathBuf::from(&args.mar_file),
                updates: updates.clone(),
            }))
            .service(update_mar)
            .service(update_xml)
    })
    .bind(("0.0.0.0", 8000))?
    .run()
    .await
}
