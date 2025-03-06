//
// Copyright 2024 Formata, Inc. All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//

use std::fs;
use anyhow::Result;
use bytes::Bytes;
use stof::{pkg::PKG, SDoc};
use crate::config::registry_path;
use super::Registry;


/// System registry.
pub struct SystemRegistry {
    /// Base registry path.
    pub base_path: String,
}
impl SystemRegistry {
    /// Create a new system registry.
    pub fn new(config: &SDoc) -> Self {
        let base_path = registry_path(config);
        Self {
            base_path,
        }
    }
}
impl Registry for SystemRegistry {
    /// Publish a package to this registry.
    fn publish(&mut self, path: &str, overwrite: bool, bytes: Bytes) -> Result<bool> {
        let dir_path = format!("{}/{}", self.base_path, path.trim_end_matches(".pkg"));
        fs::create_dir_all(&dir_path)?;

        // Create a pkg file in the directory for quick GET operations
        let pkg_path = format!("{dir_path}/__pkg__.pkg");
        let exists = fs::exists(&pkg_path);
        if exists.is_err() || (!overwrite && exists.unwrap()) {
            return Ok(false);
        }
        fs::write(&pkg_path, bytes)?;

        // Unzip the package bytes into the directory
        PKG::unzip_file(&pkg_path, &dir_path);

        Ok(true)
    }

    /// Delete package from this registry.
    fn delete(&mut self, path: &str) -> Result<bool> {
        let dir_path = format!("{}/{}", self.base_path, path.trim_end_matches(".pkg"));
        
        // If the package doesn't exist, return false
        let pkg_path = format!("{dir_path}/__pkg__.pkg");
        let exists = fs::exists(&pkg_path);
        if exists.is_err() || !exists.unwrap() {
            return Ok(false);
        }

        fs::remove_dir_all(dir_path)?;
        Ok(true)
    }

    /// Get package bytes from this registry.
    fn get(&self, path: &str) -> Result<Bytes> {
        let dir_path = format!("{}/{}", self.base_path, path.trim_end_matches(".pkg"));
        let pkg_path = format!("{dir_path}/__pkg__.pkg");
        let bytes = fs::read(pkg_path)?;
        Ok(Bytes::from(bytes))
    }
}
