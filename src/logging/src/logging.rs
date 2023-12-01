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

use std::env;

use tracing::{Event, Level};
use tracing_forest::{ForestLayer, PrettyPrinter, Tag};
use tracing_stackdriver::CloudTraceConfiguration;
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::prelude::*;
use tracing_subscriber::Registry;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LoggingType {
    Forest,
    Stackdriver,
}

impl From<LoggingType> for String {
    fn from(value: LoggingType) -> Self {
        match value {
            LoggingType::Forest => "forest",
            LoggingType::Stackdriver => "stackdriver",
        }
        .to_string()
    }
}

/// Initializes global logging behavior for the 'tracing' crate.
pub fn initialize(logging_type: LoggingType) {
    let env_filter = if let Ok(v) = env::var("RUST_LOG") {
        EnvFilter::new(v)
    } else {
        EnvFilter::new("debug,hyper=warn,h2=warn,tower=warn")
    };

    match logging_type {
        LoggingType::Forest => {
            let forest_layer = ForestLayer::new(PrettyPrinter::new(), tag_parser);
            let subscriber = Registry::default().with(forest_layer).with(env_filter);
            tracing::subscriber::set_global_default(subscriber).unwrap();
        }
        LoggingType::Stackdriver => {
            let stackdriver =
                tracing_stackdriver::layer().enable_cloud_trace(CloudTraceConfiguration {
                    project_id: "riftcaller".to_string(),
                });
            let subscriber = Registry::default().with(stackdriver).with(env_filter);
            tracing::subscriber::set_global_default(subscriber).unwrap();
        }
    }
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
        _ if target.contains("raid_state") => '🔪',
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
