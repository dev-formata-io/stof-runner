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

use axum::http::{header::AUTHORIZATION, HeaderMap};
use http_auth_basic::Credentials;
use crate::{config::{get_admin, unauth_delete, unauth_exec, unauth_read, unauth_write}, server::ServerState};
use super::{can_delete, can_exec, can_read, can_write};


/// Authenticated as admin.
pub(crate) async fn auth_admin(state: &ServerState, headers: &HeaderMap, default: bool) -> bool {
    let config = state.config.lock().await;
    if let Some(admin) = get_admin(&config) {
        if let Some(authorization) = headers.get(AUTHORIZATION) {
            if let Ok(credentials) = Credentials::from_header(authorization.to_str().unwrap().to_string()) {
                let user = credentials.user_id;
                let pass = credentials.password;
                if user == admin.0 && pass == admin.1 {
                    return true;
                }
            }
        }
    }
    default
}


/// Authenticate a read request.
pub(crate) async fn auth_read(state: &ServerState, headers: &HeaderMap) -> bool {
    let mut config = state.config.lock().await;
    if let Some(admin) = get_admin(&config) {
        if let Some(authorization) = headers.get(AUTHORIZATION) {
            if let Ok(credentials) = Credentials::from_header(authorization.to_str().unwrap().to_string()) {
                let user = credentials.user_id;
                let pass = credentials.password;
                if user == admin.0 && pass == admin.1 {
                    return true;
                } else {
                    let mut users = state.users.lock().await;
                    return can_read(&mut users, &user, &pass);
                }
            }
        }
        unauth_read(&mut config)
    } else {
        // No admin specified, so all requests are valid
        true
    }
}


/// Authenticate a write request.
pub(crate) async fn auth_write(state: &ServerState, headers: &HeaderMap, path: &str) -> bool {
    let mut config = state.config.lock().await;
    if let Some(admin) = get_admin(&config) {
        if let Some(authorization) = headers.get(AUTHORIZATION) {
            if let Ok(credentials) = Credentials::from_header(authorization.to_str().unwrap().to_string()) {
                let user = credentials.user_id;
                let pass = credentials.password;
                if user == admin.0 && pass == admin.1 {
                    return true;
                } else {
                    let mut users = state.users.lock().await;
                    return can_write(&mut users, &user, &pass, path);
                }
            }
        }
        unauth_write(&mut config)
    } else {
        // No admin specified, so all requests are valid
        true
    }
}


/// Authenticate a delete request.
pub(crate) async fn auth_delete(state: &ServerState, headers: &HeaderMap, path: &str) -> bool {
    let mut config = state.config.lock().await;
    if let Some(admin) = get_admin(&config) {
        if let Some(authorization) = headers.get(AUTHORIZATION) {
            if let Ok(credentials) = Credentials::from_header(authorization.to_str().unwrap().to_string()) {
                let user = credentials.user_id;
                let pass = credentials.password;
                if user == admin.0 && pass == admin.1 {
                    return true;
                } else {
                    let mut users = state.users.lock().await;
                    return can_delete(&mut users, &user, &pass, path);
                }
            }
        }
        unauth_delete(&mut config)
    } else {
        // No admin specified, so all requests are valid
        true
    }
}


/// Authenticate an execution request.
pub(crate) async fn auth_exec(state: &ServerState, headers: &HeaderMap) -> bool {
    let mut config = state.config.lock().await;
    if let Some(admin) = get_admin(&config) {
        if let Some(authorization) = headers.get(AUTHORIZATION) {
            if let Ok(credentials) = Credentials::from_header(authorization.to_str().unwrap().to_string()) {
                let user = credentials.user_id;
                let pass = credentials.password;
                if user == admin.0 && pass == admin.1 {
                    return true;
                } else {
                    let mut users = state.users.lock().await;
                    return can_exec(&mut users, &user, &pass);
                }
            }
        }
        unauth_exec(&mut config)
    } else {
        // No admin specified, so all requests are valid
        true
    }
}
