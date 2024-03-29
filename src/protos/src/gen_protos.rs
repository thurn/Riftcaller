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

//! Helper for generating Rust source code from protocol buffer definitions.
//!
//! Proto compilation requires that the $PROTOC and $PROTOC_INCLUDE environment
//! variables be set. For example if protoc is installed via Homebrew for OSX,
//! this might mean:
//
//! - PROTOC="/opt/homebrew/bin/protoc"
//! - PROTOC_INCLUDE="/opt/homebrew/include"

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Building rust protocol buffers");
    tonic_build::configure()
        .build_client(false)
        .type_attribute(
            "riftcaller.GameObjectIdentifier",
            "#[derive(Eq, Hash, Copy, Ord, PartialOrd)]",
        )
        .type_attribute("riftcaller.DeckIdentifier", "#[derive(Eq, Hash, Copy, Ord, PartialOrd)]")
        .type_attribute("riftcaller.GameIdentifier", "#[derive(Eq, Hash, Copy, Ord, PartialOrd)]")
        .type_attribute("riftcaller.CardIdentifier", "#[derive(Eq, Hash, Copy, Ord, PartialOrd)]")
        .type_attribute("riftcaller.InterfacePanelAddress", "#[derive(Eq, Hash)]")
        .type_attribute("riftcaller.InterfacePanelAddress.address_type", "#[derive(Eq, Hash)]")
        .type_attribute(
            "riftcaller.GameObjectIdentifier.id",
            "#[derive(Eq, Hash, Copy, Ord, PartialOrd)]",
        )
        .out_dir("src/protos/src")
        .compile(&["proto/riftcaller.proto"], &["proto/"])?;
    Ok(())
}
