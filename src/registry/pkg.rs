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

use stof::{pkg::PKG, Format};


/// Registry PKG format.
pub struct RPKG {
    pub pkg: PKG,
    pub base_path: String,
}
impl RPKG {
    pub fn new(registry_path: &str) -> Self {
        Self {
            pkg: Default::default(),
            base_path: registry_path.to_owned(),
        }
    }
}
impl Format for RPKG {
    fn format(&self) -> String {
        self.pkg.format()
    }

    fn content_type(&self) -> String {
        self.pkg.content_type()
    }

    fn header_import(&self, pid: &str, doc: &mut stof::SDoc, content_type: &str, bytes: &mut bytes::Bytes, as_name: &str) -> Result<(), stof::lang::SError> {
        self.pkg.header_import(pid, doc, content_type, bytes, as_name)
    }

    fn file_import(&self, pid: &str, doc: &mut stof::SDoc, format: &str, full_path: &str, extension: &str, as_name: &str) -> Result<(), stof::lang::SError> {
        let full_path = format!("{}/{}", self.base_path, full_path.trim_start_matches("__stof__/"));
        self.pkg.file_import(pid, doc, format, &full_path, extension, as_name)
    }
}
