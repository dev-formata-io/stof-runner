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

use std::time::Duration;
use stof::{SDoc, SField, SUnits, SVal};


/// Stof Types for Config file.
const STOF_TYPES: &str = r#"
type Server {
    #[schema((value: int): bool => value > 0 && value < 10000)]
    port: int = 3030;

    #[schema((value: vec): bool => value.len() >= 4)]
    address: vec = [127, 0, 0, 1];

    // Show Stof errors in responses?
    // Set to false to return opaque errors instead.
    errors: bool = true;

    // Can execute stof?
    run_stof: bool = true;

    // Run timeout.
    run_timeout: s = 10s;
    fn timeout(): s {
        return self.run_timeout;
    }
}

type Registry {
    // Can this runner store stof interfaces?
    enabled: bool = true;

    #[schema((value: str): bool => value.len() > 0)]
    path: str = 'registry';

    users: str = '__users__.json';
}

// By default, the runner is unprotected
type Admin {
    username: str = 'admin'
    password: str = ''

    // permissions granted to any unauthenticated user
    unauth_perms: int = 0b0000;
}

type Runner {
    #[schema]
    server: Server = new Server {};
    
    #[schema]
    registry: Registry = new Registry {};

    #[schema]
    admin: Admin = new Admin {};

    #[run]
    fn run() {
        self.valid = self.schemafy(self);
    }
}

fn unauth_read(): bool { return root.admin.unauth_perms & 0b0001 > 0; }
fn unauth_write(): bool { return root.admin.unauth_perms & 0b0010 > 0; }
fn unauth_delete(): bool { return root.admin.unauth_perms & 0b0100 > 0; }
fn unauth_exec(): bool { return root.admin.unauth_perms & 0b1000 > 0; } 

#[init]
fn init() {
    root as Runner;
    root.exec();
    if (root.debug) pln(root);
}
"#;


/// Create the Stof configuration document.
pub(crate) fn load_config(file: Option<String>) -> Result<SDoc, String> {
    let mut doc;
    if let Some(file) = file {
        if let Ok(loaded) = SDoc::file(&file, "stof") {
            doc = loaded;
        } else {
            return Err(format!("'{}' does not exist", file));
        }
    } else {
        doc = SDoc::default();
    }

    let res = doc.string_import("main", "stof", STOF_TYPES, "");
    if res.is_err() {
        return Err(format!("error loading configuration types"));
    }
    if let Some(valid) = doc.get("root.valid", None) {
        if !valid.truthy() {
            return Err(format!("not a valid configuration"));
        }
    } else {
        return Err(format!("not a valid configuration"));
    }

    Ok(doc)
}


/// Server port.
pub(crate) fn server_port(config: &SDoc) -> u16 {
    let mut port = 3030;
    if let Some(port_field) = SField::field(&config.graph, "root.server.port", '.', None) {
        match &port_field.value {
            SVal::Number(num) => {
                port = num.int() as u16;
            },
            _ => {}
        }
    }
    port
}


/// Server address.
pub(crate) fn server_address(config: &SDoc) -> [u8; 4] {
    let mut ip = [127, 0, 0, 1];
    if let Some(ip_field) = SField::field(&config.graph, "root.server.address", '.', None) {
        match &ip_field.value {
            SVal::Array(vals) => {
                if vals.len() == 4 {
                    for i in 0..4 {
                        match &vals[i] {
                            SVal::Number(num) => {
                                ip[i] = num.int() as u8;
                            },
                            _ => {}
                        }
                    }
                }
            },
            _ => {}
        }
    }
    ip
}


/// Opaque errors?
/// Return true to hide specific errors from responses.
pub(crate) fn opaque_errors(config: &SDoc) -> bool {
    if let Some(error_field) = SField::field(&config.graph, "root.server.errors", '.', None) {
        match &error_field.value {
            SVal::Bool(show_errors) => {
                return !show_errors;
            },
            _ => {}
        }
    }
    true
}


/// Running Stof enabled?
pub(crate) fn run_enabled(config: &SDoc) -> bool {
    if let Some(run_field) = SField::field(&config.graph, "root.server.run_stof", '.', None) {
        match &run_field.value {
            SVal::Bool(run_enabled) => {
                return *run_enabled;
            },
            _ => {}
        }
    }
    false
}


/// Run timeout duration.
pub(crate) fn run_timeout(config: &mut SDoc) -> Duration {
    if let Ok(res) = config.call_func("root.server.timeout", None, vec![]) {
        // returns number in seconds
        match res {
            SVal::Number(num) => {
                return Duration::from_secs(num.float_with_units(SUnits::Seconds) as u64);
            },
            _ => {}
        }
    }
    Duration::from_secs(10)
}


/// Registry enabled?
pub(crate) fn registry_enabled(config: &SDoc) -> bool {
    if let Some(enabled_field) = SField::field(&config.graph, "root.registry.enabled", '.', None) {
        match &enabled_field.value {
            SVal::Bool(val) => {
                return *val;
            },
            _ => {}
        }
    }
    false
}


/// Registry path.
pub(crate) fn registry_path(config: &SDoc) -> String {
    let mut path = String::from("registry");
    if let Some(registry_path) = SField::field(&config.graph, "root.registry.path", '.', None) {
        path = registry_path.to_string();
    }
    path
}


/// Registry users file name.
pub(crate) fn registry_users_filename(config: &SDoc) -> String {
    let mut name = String::from("__users__.json");
    if let Some(users_file) = SField::field(&config.graph, "root.registry.users", '.', None) {
        name = users_file.to_string();
    }
    name
}


/// Get admin (if defined).
/// Returns admin username & password if the configuration contains an admin definition (both username and non-empty password).
pub(crate) fn get_admin(config: &SDoc) -> Option<(String, String)> {
    if let Some(username) = config.field("root.admin.username", None) {
        if let Some(password) = config.field("root.admin.password", None) {
            let user = username.to_string();
            let pass = password.to_string();
            if user.len() > 0 && pass.len() > 0 {
                return Some((user, pass));
            }
        }
    }
    None
}


/// Unauthenticated read permissions?
pub(crate) fn unauth_read(config: &mut SDoc) -> bool {
    if let Ok(res) = config.call_func("root.unauth_read", None, vec![]) {
        return res.truthy();
    }
    return false;
}


/// Unauthenticated write permissions?
pub(crate) fn unauth_write(config: &mut SDoc) -> bool {
    if let Ok(res) = config.call_func("root.unauth_write", None, vec![]) {
        return res.truthy();
    }
    return false;
}


/// Unauthenticated delete permissions?
pub(crate) fn unauth_delete(config: &mut SDoc) -> bool {
    if let Ok(res) = config.call_func("root.unauth_delete", None, vec![]) {
        return res.truthy();
    }
    return false;
}


/// Unauthenticated exec permissions?
pub(crate) fn unauth_exec(config: &mut SDoc) -> bool {
    if let Ok(res) = config.call_func("root.unauth_exec", None, vec![]) {
        return res.truthy();
    }
    return false;
}
