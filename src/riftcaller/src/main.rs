// Copyright © Riftcaller 2021-present

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//    https://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Riftcaller: An asymmetric trading card game

use std::env;

use database::firestore_database::FirestoreDatabase;
use database::sled_database::SledDatabase;
use database::Database;
use logging::LoggingType;
use protos::riftcaller::riftcaller_server::RiftcallerServer;
use server::GameService;
use tonic::codec::CompressionEncoding;
use tonic::transport::Server;
use tonic_web::GrpcWebLayer;
use tracing::warn;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    cards_all::initialize();
    let args = env::args().collect::<Vec<_>>();

    let version = if args.len() >= 3 { &args[2] } else { "unspecified" };

    let logging_type = if args.len() >= 4 && args[3].contains("stackdriver") {
        LoggingType::Stackdriver
    } else {
        LoggingType::Forest
    };
    logging::initialize(logging_type);

    let port = env::var("PORT").unwrap_or_else(|_| "80".to_string());

    if args.len() >= 2 && args[1].contains("firestore") {
        start_server(
            version,
            port,
            FirestoreDatabase::new("riftcaller").await?,
            "firestore",
            logging_type,
        )
        .await
    } else {
        start_server(version, port, SledDatabase::new("db"), "sled", logging_type).await
    }
}

async fn start_server(
    version: impl Into<String>,
    port: impl Into<String>,
    database: impl Database + 'static,
    db_name: impl Into<String>,
    logging: impl Into<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let address = format!("0.0.0.0:{}", port.into()).parse().expect("valid address");
    let server = RiftcallerServer::new(GameService { database })
        .send_compressed(CompressionEncoding::Gzip)
        .accept_compressed(CompressionEncoding::Gzip);

    warn!(
        "{} server version '{}' listening on '{}' with '{}' database and '{}' logging",
        if cfg!(debug_assertions) { "Debug" } else { "Release" },
        version.into(),
        address,
        db_name.into(),
        logging.into()
    );

    Server::builder()
        .trace_fn(|_| tracing::info_span!(">>>"))
        .accept_http1(true)
        .layer(GrpcWebLayer::new())
        .add_service(server)
        .serve(address)
        .await?;

    Ok(())
}
