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
use stof::{SData, SDoc, SField, SVal};
use crate::config::registry_path;
pub(crate) mod api;


const METRICS_INTERFACE: &str = r#"
Config: {
    save_path: 'registry/__metrics__.bstof'
    last_modified: Time.now()

    fn save() {
        let bytes = blobify(root, 'bstof');
        fs.write_blob(self.save_path, bytes);
        self.last_modified = Time.now();
    }
    fn trySave() {
        let now = Time.now();
        let diff = now - self.last_modified;
        if (diff > 10min) self.save();
    }
}
Server: {
    Run: {
        count: 0

        fn getCount(): int {
            return self.count;
        }
        fn increment() {
            self.count += 1;
            root.Config.trySave();
        }
    }
}
Registry: {
    Packages: {
        count: 0
        
        fn getCount(): int {
            return self.count;
        }
        fn increment() {
            self.count += 1;
            root.Config.trySave();
        }
        fn deincrement() {
            let current = self.count;
            if (current > 0) current -= 1;
            self.count = current;
            root.Config.trySave();
        }
    }
    Downloads: {
        total: 0

        fn getTotalDownloadCount(): int {
            return self.total;
        }
        fn getDownloadCount(package: str): int {
            let path = package.replace('/', '.') + '.downloads';
            return self.at(path).or(0);
        }
        fn increment(package: str) {
            self.total += 1;
            let path = package.replace('/', '.') + '.downloads';
            let current = self.at(path).or(0);
            self.set(path, current + 1);
            root.Config.trySave();
        }
    }
}
"#;


/// Load metrics file.
pub(crate) fn load_metrics(config: &SDoc) -> SDoc {
    let registry_name = registry_path(config);
    let metrics_file_path = format!("{}/__metrics__.bstof", registry_name);

    if let Ok(exists) = fs::exists(&metrics_file_path) {
        if exists {
            if let Ok(doc) = SDoc::file(&metrics_file_path, "bstof") {
                return doc;
            }
        }
    }

    let mut doc = SDoc::default();
    let _ = doc.string_import("main", "stof", METRICS_INTERFACE, "");

    if let Some(field_ref) = SField::field_ref(&doc.graph, "root.Config.save_path", '.', None) {
        if let Some(field) = SData::get_mut::<SField>(&mut doc.graph, &field_ref) {
            field.value = metrics_file_path.into();
        }
    }
    doc
}


/// Get server run count.
pub(crate) fn server_run_count(metrics: &mut SDoc) -> i64 {
    if let Ok(res) = metrics.call_func("root.Server.Run.getCount", None, vec![]) {
        match res {
            SVal::Number(num) => {
                return num.int();
            },
            _ => {}
        }
    }
    0
}


/// Increment server run count.
pub(crate) fn increment_server_run_count(metrics: &mut SDoc) -> bool {
    if let Ok(_) = metrics.call_func("root.Server.Run.increment", None, vec![]) {
        return true;
    }
    false
}


/// Registry packages count.
pub(crate) fn registry_packages_count(metrics: &mut SDoc) -> i64 {
    if let Ok(res) = metrics.call_func("root.Registry.Packages.getCount", None, vec![]) {
        match res {
            SVal::Number(num) => {
                return num.int();
            },
            _ => {}
        }
    }
    0
}


/// Registry packages increment count.
pub(crate) fn registry_packages_increment_count(metrics: &mut SDoc) -> bool {
    if let Ok(_) = metrics.call_func("root.Registry.Packages.increment", None, vec![]) {
        return true;
    }
    false
}


/// Registry packages deincrement count.
pub(crate) fn registry_packages_deincrement_count(metrics: &mut SDoc) -> bool {
    if let Ok(_) = metrics.call_func("root.Registry.Packages.deincrement", None, vec![]) {
        return true;
    }
    false
}


/// Registry downloads total count.
pub(crate) fn registry_downloads_total_count(metrics: &mut SDoc) -> i64 {
    if let Ok(res) = metrics.call_func("root.Registry.Downloads.getTotalDownloadCount", None, vec![]) {
        match res {
            SVal::Number(num) => {
                return num.int();
            },
            _ => {}
        }
    }
    0
}


/// Registry downloads package count.
pub(crate) fn registry_downloads_count(metrics: &mut SDoc, package: &str) -> i64 {
    if let Ok(res) = metrics.call_func("root.Registry.Downloads.getDownloadCount", None, vec![package.into()]) {
        match res {
            SVal::Number(num) => {
                return num.int();
            },
            _ => {}
        }
    }
    0
}


/// Registry downloads increment package count.
pub(crate) fn registry_downloads_increment_count(metrics: &mut SDoc, package: &str) -> bool {
    if let Ok(_) = metrics.call_func("root.Registry.Downloads.increment", None, vec![package.into()]) {
        return true;
    }
    false
}
