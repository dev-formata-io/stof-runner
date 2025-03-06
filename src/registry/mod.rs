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

use anyhow::Result;
use bytes::Bytes;
pub mod system;
pub mod pkg;
pub mod api;


/// Registry trait.
pub trait Registry: Send + Sync {
    /// Publish a package to this registry.
    fn publish(&mut self, path: &str, overwrite: bool, bytes: Bytes) -> Result<bool>;

    /// Delete a package from this registry.
    fn delete(&mut self, path: &str) -> Result<bool>;

    /// Get a package from this registry.
    fn get(&self, path: &str) -> Result<Bytes>;
}
