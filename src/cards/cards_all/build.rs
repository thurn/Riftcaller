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

//! Generates cards_all.rs. This used to be automatic via the 'linkme' crate,
//! but new versions of Rust have caused a bunch of crazy problems with it.

use std::{env, fs};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, LineWriter, Write};
use std::path::Path;

use anyhow::Result;
use regex::Regex;
use walkdir::WalkDir;

fn main() -> Result<()> {
    println!("Generating cards_all.rs in {:?}", env::current_dir().unwrap());

    // crate name -> vec of function names
    let mut functions = HashMap::new();
    for e in WalkDir::new("..") {
        let entry = e?;
        let re = Regex::new(r"../(?P<module>\w+)/src/(?P<file>\w+).rs")?;

        if let Some(captures) = re.captures(entry.path().to_str().expect("str")) {
            let found = find_functions(entry.path())?;
            if !found.is_empty() {
                functions.insert(format!("{}::{}", &captures["module"], &captures["file"]), found);
            }
        }
    }

    let out_path = Path::new("../cards_all/src/cards_all.rs");
    if out_path.exists() {
        fs::remove_file(out_path)?;
    }
    let mut file = LineWriter::new(File::create(out_path)?);
    writeln!(file, "//! GENERATED CODE - DO NOT MODIFY\n")?;

    writeln!(file, "use rules::DEFINITIONS;\n")?;

    let mut modules = functions
        .iter()
        .filter(|(_, list)| !list.is_empty())
        .map(|(module, _)| module.clone())
        .collect::<Vec<_>>();
    modules.sort();

    writeln!(file, "\npub fn initialize() {{")?;
    for module in &modules {
        if let Some(list) = functions.get(module) {
            for function in list {
                writeln!(file, "    DEFINITIONS.insert({module}::{function});")?;
            }
        }
    }
    writeln!(file, "}}")?;

    Ok(())
}

fn find_functions(path: impl AsRef<Path>) -> Result<Vec<String>> {
    let mut result = vec![];
    let re = Regex::new(r"pub fn (?P<name>\w+)\(.*\) -> CardDefinition")?;
    for l in BufReader::new(File::open(path)?).lines() {
        let line = l?;
        if let Some(captures) = re.captures(&line) {
            result.push(captures["name"].to_string());
        }
    }
    Ok(result)
}
