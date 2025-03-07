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

use axum::{extract::State, http::{header::CONTENT_TYPE, HeaderMap, StatusCode}, response::IntoResponse};
use bytes::Bytes;
use stof::{SDoc, SVal};
use crate::{response::StofResponse, server::ServerState};
use super::{admin_delete_user, admin_set_user, auth::auth_admin};


/// Create/update a user.
pub(crate) async fn admin_set_user_handler(State(state): State<ServerState>, headers: HeaderMap, body: Bytes) -> impl IntoResponse {
    if !auth_admin(&state, &headers, false).await {
        return StofResponse::error(StatusCode::FORBIDDEN, "access denied");
    }

    let mut content_type = String::from("stof");
    if let Some(ctype) = headers.get(CONTENT_TYPE) {
        content_type = ctype.to_str().unwrap().to_owned();
    }
    if let Ok(doc) = SDoc::bytes(body, &content_type) {
        if let Some(user) = doc.field("root.username", None) {
            if let Some(pass) = doc.field("root.password", None) {
                if let Some(perms) = doc.field("root.perms", None) {
                    let mut scope = String::default();
                    if let Some(scope_field) = doc.field("root.scope", None) {
                        scope = scope_field.to_string();
                    }
                    match &perms.value {
                        SVal::Number(num) => {
                            let perms = num.int();
                            let mut users = state.users.lock().await;
                            if admin_set_user(&mut users, &user.to_string(), &pass.to_string(), perms, &scope) {
                                return StofResponse::msg(StatusCode::OK, "set user");
                            }
                        },
                        _ => {}
                    }
                }
            }
        }
    }
    StofResponse::error(StatusCode::BAD_REQUEST, "not a valid user body")
}


/// Delete a user.
pub(crate) async fn admin_delete_user_handler(State(state): State<ServerState>, headers: HeaderMap, body: Bytes) -> impl IntoResponse {
    if !auth_admin(&state, &headers, false).await {
        return StofResponse::error(StatusCode::FORBIDDEN, "access denied");
    }
    
    let mut content_type = String::from("stof");
    if let Some(ctype) = headers.get(CONTENT_TYPE) {
        content_type = ctype.to_str().unwrap().to_owned();
    }
    if let Ok(doc) = SDoc::bytes(body, &content_type) {
        if let Some(user) = doc.field("root.username", None) {
            let mut users = state.users.lock().await;
            if admin_delete_user(&mut users, &user.to_string()) {
                return StofResponse::msg(StatusCode::OK, "deleted user");
            }
        }
    }
    StofResponse::error(StatusCode::BAD_REQUEST, "not a valid user body")
}
