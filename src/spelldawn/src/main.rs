// Copyright © Spelldawn 2021-present

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//    https://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Spelldawn: An asymmetric trading card game

use std::{env, thread};

use database::firestore_database::FirestoreDatabase;
use database::sled_database::SledDatabase;
use database::Database;
use protos::spelldawn::spelldawn_server::SpelldawnServer;
use server::GameService;
use signal_hook::consts::SIGTERM;
use signal_hook::iterator::Signals;
use tonic::codec::CompressionEncoding;
use tonic::transport::Server;
use tonic_web::GrpcWebLayer;
use tracing::{warn, Event, Level};
use tracing_forest::{ForestLayer, PrettyPrinter, Tag};
use tracing_stackdriver::CloudTraceConfiguration;
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::prelude::*;
use tracing_subscriber::Registry;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    cards_all::initialize();
    let args = env::args().collect::<Vec<_>>();

    let mut signals = Signals::new([SIGTERM])?;
    thread::spawn(move || {
        for _ in signals.forever() {
            warn!("Received SIGTERM");
            println!("Received SIGTERM");
        }
    });

    let filter = if let Ok(v) = env::var("RUST_LOG") {
        EnvFilter::new(v)
    } else {
        EnvFilter::new("debug,hyper=warn,h2=warn,tower=warn")
    };

    let version = if args.len() >= 3 { &args[2] } else { "unspecified" };

    let logging = if args.len() >= 4 && args[3].contains("stackdriver") {
        let stackdriver = tracing_stackdriver::layer()
            .enable_cloud_trace(CloudTraceConfiguration { project_id: "spelldawn".to_string() });
        let subscriber = Registry::default().with(stackdriver).with(filter);
        tracing::subscriber::set_global_default(subscriber).unwrap();
        "stackdriver"
    } else {
        let forest_layer = ForestLayer::new(PrettyPrinter::new(), tag_parser);
        let subscriber = Registry::default().with(forest_layer).with(filter);
        tracing::subscriber::set_global_default(subscriber).unwrap();
        "tracing-forest"
    };

    let port = env::var("PORT").unwrap_or_else(|_| "80".to_string());

    if args.len() >= 2 && args[1].contains("firestore") {
        start_server(
            version,
            port,
            FirestoreDatabase::new("spelldawn").await?,
            "firestore",
            logging,
        )
        .await
    } else {
        start_server(version, port, SledDatabase::new("db"), "sled", logging).await
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
    let server = SpelldawnServer::new(GameService { database })
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

fn tag_parser(event: &Event) -> Option<Tag> {
    let target = event.metadata().target();
    let level = *event.metadata().level();
    let icon = match target {
        _ if level == Level::ERROR => '🚨',
        _ if level == Level::WARN => '🚧',
        _ if target.contains("rules") => '🎴',
        _ if target.contains("tutorial") => '🎓',
        _ if target.contains("server") => '💻',
        _ if target.contains("actions") => '🎬',
        _ if target.contains("raids") => '🔪',
        _ => match level {
            Level::TRACE => '📍',
            Level::DEBUG => '📝',
            _ => '💡',
        },
    };

    let mut builder = Tag::builder().level(level).icon(icon);
    if icon == '📝' || icon == '💡' || icon == '📍' {
        builder = builder.prefix(target);
    }

    Some(builder.build())
}
