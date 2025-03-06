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

use std::collections::BTreeMap;
use axum::{extract::{Path, Query, State}, http::{HeaderMap, StatusCode}, response::IntoResponse};
use bytes::Bytes;
use crate::{config::registry_enabled, response::StofResponse, server::ServerState, users::auth::{auth_delete, auth_read, auth_write}};


/// Publish to this registry handler.
pub(crate) async fn publish_registry_handler(State(state): State<ServerState>, Path(path): Path<String>, Query(query): Query<BTreeMap<String, String>>, headers: HeaderMap, body: Bytes) -> impl IntoResponse {
    if !auth_write(&state, &headers, &path).await {
        return StofResponse::error(StatusCode::FORBIDDEN, "access denied");
    }
    if path.split('/').collect::<Vec<&str>>().len() < 2 {
        return StofResponse::error(StatusCode::BAD_REQUEST, "package directory not found");
    }

    {
        let config = state.config.lock().await;
        if !registry_enabled(&config) {
            return StofResponse::error(StatusCode::NOT_IMPLEMENTED, "registry is not available");
        }
    }

    let mut overwrite = true;
    if let Some(q_overwrite) = query.get("overwrite") {
        overwrite = q_overwrite == "true";
    }

    let mut registry = state.registry.lock().await;
    if let Ok(res) = registry.publish(&path, overwrite, body) {
        if res {
            return StofResponse::msg(StatusCode::OK, "package created");
        }
    }
    StofResponse::error(StatusCode::BAD_REQUEST, "package not created")
}


/// Delete a package from this registry handler.
pub(crate) async fn delete_registry_handler(State(state): State<ServerState>, Path(path): Path<String>, headers: HeaderMap) -> impl IntoResponse {
    if !auth_delete(&state, &headers, &path).await {
        return StofResponse::error(StatusCode::FORBIDDEN, "access denied");
    }
    if path.split('/').collect::<Vec<&str>>().len() < 2 {
        return StofResponse::error(StatusCode::BAD_REQUEST, "package directory not found");
    }

    {
        let config = state.config.lock().await;
        if !registry_enabled(&config) {
            return StofResponse::error(StatusCode::NOT_IMPLEMENTED, "registry is not available");
        }
    }

    let mut registry = state.registry.lock().await;
    if let Ok(res) = registry.delete(&path) {
        if res {
            return StofResponse::msg(StatusCode::OK, "package removed");
        }
    }
    StofResponse::error(StatusCode::BAD_REQUEST, "package not found")
}


/// Get a package from this registry handler.
pub(crate) async fn get_registry_handler(State(state): State<ServerState>, Path(path): Path<String>, headers: HeaderMap) -> impl IntoResponse {
    if !auth_read(&state, &headers).await {
        return StofResponse::error(StatusCode::FORBIDDEN, "access denied");
    }
    if path.split('/').collect::<Vec<&str>>().len() < 2 {
        return StofResponse::error(StatusCode::BAD_REQUEST, "package directory not found");
    }

    {
        let config = state.config.lock().await;
        if !registry_enabled(&config) {
            return StofResponse::error(StatusCode::NOT_IMPLEMENTED, "registry is not available");
        }
    }

    let registry = state.registry.lock().await;
    if let Ok(bytes) = registry.get(&path) {
        return StofResponse::bytes(StatusCode::OK, bytes);
    }
    StofResponse::error(StatusCode::BAD_REQUEST, "package not found")
}
