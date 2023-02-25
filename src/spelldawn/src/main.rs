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

use std::env;

use database::firestore_database::FirestoreDatabase;
use database::sled_database::SledDatabase;
use database::Database;
use protos::spelldawn::spelldawn_server::SpelldawnServer;
use server::GameService;
use tonic::codec::CompressionEncoding;
use tonic::transport::Server;
use tonic_web::GrpcWebLayer;
use tracing::{info, warn, Event, Level};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_forest::{ForestLayer, PrettyPrinter, Tag};
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::prelude::*;
use tracing_subscriber::Registry;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    cards_all::initialize();
    let args = env::args().collect::<Vec<_>>();

    let file_appender = tracing_appender::rolling::hourly("log", "spelldawn.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    let bunyan = BunyanFormattingLayer::new("spelldawn".to_string(), non_blocking);
    let filter = if let Ok(v) = env::var("RUST_LOG") {
        EnvFilter::new(v)
    } else {
        EnvFilter::new("debug,hyper=warn,h2=warn,tower=warn")
    };
    let forest_layer = ForestLayer::new(PrettyPrinter::new(), tag_parser);

    let subscriber =
        Registry::default().with(JsonStorageLayer).with(bunyan).with(forest_layer).with(filter);
    tracing::subscriber::set_global_default(subscriber).unwrap();

    if args.len() >= 2 && args[1] == "--firestore" {
        info!("Using firestore database");
        start_server(FirestoreDatabase::new("spelldawn").await?).await
    } else {
        info!("Using sled database");
        start_server(SledDatabase).await
    }
}

async fn start_server(database: impl Database + 'static) -> Result<(), Box<dyn std::error::Error>> {
    let address = "0.0.0.0:80".parse().expect("valid address");
    let server = SpelldawnServer::new(GameService { database })
        .send_compressed(CompressionEncoding::Gzip)
        .accept_compressed(CompressionEncoding::Gzip);

    warn!("Server listening on {}", address);
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
