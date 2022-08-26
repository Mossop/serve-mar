/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::{
    fs::{metadata, File},
    io::{self, BufRead, BufReader, ErrorKind, Read},
    path::Path,
};

use hmac_sha512::Hash;
use ini::Ini;
use mar::Mar;
use serde::Serialize;
use xml_serde::{to_string_custom, Options};

const BUFFER_SIZE: usize = 5 * 1024 * 1024;

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum UpdateType {
    Minor,
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PatchType {
    Complete,
    Partial,
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum HashFunction {
    Sha512,
}

#[derive(Clone, Serialize)]
pub struct Patch {
    #[serde(rename = "$attr:type")]
    pub patch_type: PatchType,
    #[serde(rename = "$attr:URL")]
    pub url: String,
    #[serde(rename = "$attr:hashFunction")]
    pub hash_function: HashFunction,
    #[serde(rename = "$attr:hashValue")]
    pub hash_value: String,
    #[serde(rename = "$attr:size")]
    pub size: u64,
}

impl Patch {
    pub fn from_file<P: AsRef<Path>>(path: P) -> io::Result<Patch> {
        let stat = metadata(&path)?;

        if !stat.is_file() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Patches must be files",
            ));
        }

        let mut reader = BufReader::new(File::open(path)?);
        let mut buffer = [0_u8; BUFFER_SIZE];
        let mut hasher = Hash::new();

        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(len) => {
                    hasher.update(&buffer[0..len]);
                }
                Err(e) => {
                    if e.kind() != ErrorKind::Interrupted {
                        return Err(e);
                    }
                }
            }
        }

        Ok(Patch {
            patch_type: PatchType::Complete,
            url: "http://localhost:8000/update.mar".to_string(),
            hash_function: HashFunction::Sha512,
            hash_value: hex::encode(hasher.finalize()),
            size: stat.len(),
        })
    }
}

#[derive(Clone, Serialize)]
pub struct Update {
    #[serde(rename = "$attr:type")]
    pub update_type: UpdateType,
    #[serde(rename = "$attr:displayVersion")]
    pub display_version: String,
    #[serde(rename = "$attr:appVersion")]
    pub app_version: String,
    #[serde(rename = "$attr:platformVersion")]
    pub platform_version: String,
    #[serde(rename = "$attr:buildID")]
    pub build_id: String,
    #[serde(rename = "patch")]
    pub patches: Vec<Patch>,
}

impl Update {
    pub fn from_mar<P: AsRef<Path>>(path: P) -> io::Result<Update> {
        let mut patch = Patch::from_file(&path)?;
        let mut version = "2000.0a1".to_string();
        let mut build_id = "21181002100236".to_string();

        let mut mar = Mar::from_path(&path)?;
        for item in mar.files()? {
            let item = item?;
            if item.name == "updatev3.manifest" {
                let reader = BufReader::new(mar.read(&item)?);
                if let Some(Ok(line)) = reader.lines().next() {
                    if line == "type \"partial\"" {
                        patch.patch_type = PatchType::Partial;
                    }
                }
            } else if item.name == "application.ini"
                || item.name == "Contents/Resources/application.ini"
            {
                if let Ok(ini) = Ini::read_from(&mut mar.read(&item)?) {
                    if let Some(val) = ini.get_from(Some("App"), "Version") {
                        version = val.to_string();
                    }

                    if let Some(val) = ini.get_from(Some("App"), "BuildID") {
                        build_id = val.to_string();
                    }
                }
            }
        }

        Ok(Update {
            update_type: UpdateType::Minor,
            display_version: version.clone(),
            app_version: version.clone(),
            platform_version: version,
            build_id,
            patches: vec![patch],
        })
    }
}

#[derive(Serialize)]
struct UpdateListInner<'a> {
    #[serde(rename = "update")]
    pub updates: &'a Vec<Update>,
}

#[derive(Serialize)]
struct UpdateListOuter<'a> {
    #[serde(rename = "updates")]
    pub inner: UpdateListInner<'a>,
}

#[derive(Clone)]
pub struct Updates {
    pub updates: Vec<Update>,
}

impl Updates {
    pub fn from_mar<P: AsRef<Path>>(path: P) -> io::Result<Updates> {
        Ok(Updates {
            updates: vec![Update::from_mar(path)?],
        })
    }

    pub fn serialize(&self) -> Result<String, io::Error> {
        let outer = UpdateListOuter {
            inner: UpdateListInner {
                updates: &self.updates,
            },
        };

        to_string_custom(
            &outer,
            Options {
                include_schema_location: false,
            },
        )
        .map_err(|e| io::Error::new(ErrorKind::InvalidData, format!("{}", e)))
    }
}
