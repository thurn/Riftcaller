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

use protos::spelldawn::spelldawn_server::SpelldawnServer;
use server::requests::GameService;
use tonic::transport::Server;
use tracing::{warn, Event, Level};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_forest::{ForestLayer, PrettyPrinter, Tag};
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::prelude::*;
use tracing_subscriber::Registry;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    cards_all::initialize();

    let file_appender = tracing_appender::rolling::hourly("log", "spelldawn.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    let bunyan = BunyanFormattingLayer::new("spelldawn".to_string(), non_blocking);
    let filter = if let Ok(v) = env::var("RUST_LOG") {
        EnvFilter::new(v)
    } else {
        EnvFilter::new("debug,hyper=warn")
    };
    let forest_layer = ForestLayer::new(PrettyPrinter::new(), tag_parser);

    let subscriber =
        Registry::default().with(JsonStorageLayer).with(bunyan).with(forest_layer).with(filter);
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let address = "0.0.0.0:80".parse().expect("valid address");
    let server = SpelldawnServer::new(GameService {
        // To print responses:
        // response_interceptor: Some(|response| eprintln!("{}", Summary::summarize(response)))
        response_interceptor: None,
    })
    .send_gzip()
    .accept_gzip();
    let service = tonic_web::config().enable(server);

    warn!("Server listening on {}", address);
    Server::builder().accept_http1(true).add_service(service).serve(address).await?;

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
