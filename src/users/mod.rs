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

pub(crate) mod auth;
pub(crate) mod api;

use stof::{SData, SDoc, SField};
use crate::config::{registry_path, registry_users_filename};


const USERS_INTERFACE: &str = r#"
// make sure the Users root exists
root Users: {}

type User {
    username: str;

    #[private]
    password: str;

    // 0b(exec, delete, write, read)
    perms: int = 0b0000;

    // modify scope
    // if set, this user can only modify this registry within this scope
    scope: str = '';

    fn authenticated(password: str): bool {
        return self.password == password;
    }
    fn can_read(): bool {
        return self.perms & 0b0001 > 0;
    }
    fn can_write(): bool {
        return self.perms & 0b0010 > 0;
    }
    fn can_delete(): bool {
        return self.perms & 0b0100 > 0;
    }
    fn can_exec(): bool {
        return self.perms & 0b1000 > 0;
    }
    fn can_modify_scope(path: str): bool {
        let user_scope = self.scope;
        if (user_scope.len() < 1) return true;

        let scope = path.split('/').first();
        if (scope.startsWith('@')) scope = scope.substring(1);
        return scope == user_scope;
    }
}

obj Admin: {
    // default export JSON path
    export_json_path: 'registry/__users__.json'

    // set a user
    fn set_user(username: str, password: str, perms: int = 0b1111, scope: str = ''): bool {
        Users.removeField(username, true);
        return Users.set(username, new User {
            username: username,
            password: password,
            perms: perms,
            scope: scope,
        });
    }

    // delete a user by username
    fn delete_user(username: str): bool {
        return Users.removeField(username, true);
    }

    // export users to a json file
    fn export_json_users(path: str = root.Admin.export_json_path) {
        let json = stringify(Users, 'json');
        fs.write(path, json);
    }
}

// authenticate a user by username, returning the user if present
fn authenticate(username: str, password: str): User {
    let user: User = Users.at(username);
    if (user && user.authenticated(password)) {
        return user;
    }
    return null;
}

// can this user read?
fn can_read(username: str, password: str): bool {
    let user = self.authenticate(username, password);
    return user && user.can_read();
}

// can this user write?
fn can_write(username: str, password: str, path: str = ''): bool {
    let user = self.authenticate(username, password);
    return user && user.can_write() && (path.len() < 1 || user.can_modify_scope(path));
}

// can this user delete?
fn can_delete(username: str, password: str, path: str = ''): bool {
    let user = self.authenticate(username, password);
    return user && user.can_delete() && (path.len() < 1 || user.can_modify_scope(path));
}

// can this user exec?
fn can_exec(username: str, password: str): bool {
    let user = self.authenticate(username, password);
    return user && user.can_exec();
}
"#;


/// Load the stof users document.
pub(crate) fn load_users(config: &SDoc) -> SDoc {
    let registry_name = registry_path(config);
    let users_name = registry_users_filename(config);
    let users_file_path = format!("{}/{}", registry_name, users_name);

    let mut doc = SDoc::default();
    let _ = doc.file_import("main", "json", &users_file_path, "json", "Users");
    let _ = doc.string_import("main", "stof", USERS_INTERFACE, "");

    if let Some(field_ref) = SField::field_ref(&doc.graph, "root.Admin.export_json_path", '.', None) {
        if let Some(field) = SData::get_mut::<SField>(&mut doc.graph, &field_ref) {
            field.value = users_file_path.into();
        }
    }
    doc
}


/// ADMIN export users file.
pub(crate) fn admin_export_users(users: &mut SDoc) {
    let _ = users.call_func("root.Admin.export_json_users", None, vec![]);
}


/// ADMIN create a new user.
pub(crate) fn admin_set_user(users: &mut SDoc, user: &str, pass: &str, perms: i64, scope: &str) -> bool {
    if let Ok(res) = users.call_func("root.Admin.set_user", None, vec![user.into(), pass.into(), perms.into(), scope.into()]) {
        admin_export_users(users);
        return res.truthy();
    }
    false
}


/// ADMIN delete a user.
pub(crate) fn admin_delete_user(users: &mut SDoc, user: &str) -> bool {
    if let Ok(res) = users.call_func("root.Admin.delete_user", None, vec![user.into()]) {
        admin_export_users(users);
        return res.truthy();
    }
    false
}


/// Can read?
pub(crate) fn can_read(users: &mut SDoc, user: &str, pass: &str) -> bool {
    if let Ok(res) = users.call_func("root.can_read", None, vec![user.into(), pass.into()]) {
        return res.truthy();
    }
    false
}


/// Can write?
pub(crate) fn can_write(users: &mut SDoc, user: &str, pass: &str, path: &str) -> bool {
    if let Ok(res) = users.call_func("root.can_write", None, vec![user.into(), pass.into(), path.into()]) {
        return res.truthy();
    }
    false
}


/// Can delete?
pub(crate) fn can_delete(users: &mut SDoc, user: &str, pass: &str, path: &str) -> bool {
    if let Ok(res) = users.call_func("root.can_delete", None, vec![user.into(), pass.into(), path.into()]) {
        return res.truthy();
    }
    false
}


/// Can exec?
pub(crate) fn can_exec(users: &mut SDoc, user: &str, pass: &str) -> bool {
    if let Ok(res) = users.call_func("root.can_exec", None, vec![user.into(), pass.into()]) {
        return res.truthy();
    }
    false
}
