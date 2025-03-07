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

use axum::{extract::{Path, State}, http::{HeaderMap, StatusCode}, response::IntoResponse};
use crate::{response::StofResponse, server::ServerState, users::auth::auth_admin};
use super::{registry_downloads_count, registry_downloads_total_count, registry_packages_count, server_run_count};


/// Get server run count handler.
pub(crate) async fn get_server_run_count_handler(State(state): State<ServerState>, headers: HeaderMap) -> impl IntoResponse {
    if !auth_admin(&state, &headers, true).await {
        return StofResponse::error(StatusCode::FORBIDDEN, "access denied");
    }

    let count;
    {
        let mut metrics = state.metrics.lock().await;
        count = server_run_count(&mut metrics);
    }

    let stof = format!("int count: {}", count);
    StofResponse::stof(StatusCode::OK, &stof)
}


/// Get registry packages count handler.
pub(crate) async fn get_packages_count_handler(State(state): State<ServerState>, headers: HeaderMap) -> impl IntoResponse {
    if !auth_admin(&state, &headers, true).await {
        return StofResponse::error(StatusCode::FORBIDDEN, "access denied");
    }

    let count;
    {
        let mut metrics = state.metrics.lock().await;
        count = registry_packages_count(&mut metrics);
    }

    let stof = format!("int count: {}", count);
    StofResponse::stof(StatusCode::OK, &stof)
}


/// Get registry total downloads count handler.
pub(crate) async fn get_total_downloads_count_handler(State(state): State<ServerState>, headers: HeaderMap) -> impl IntoResponse {
    if !auth_admin(&state, &headers, true).await {
        return StofResponse::error(StatusCode::FORBIDDEN, "access denied");
    }

    let count;
    {
        let mut metrics = state.metrics.lock().await;
        count = registry_downloads_total_count(&mut metrics);
    }

    let stof = format!("int count: {}", count);
    StofResponse::stof(StatusCode::OK, &stof)
}


/// Get registry downloads count handler.
pub(crate) async fn get_downloads_count_handler(State(state): State<ServerState>, Path(path): Path<String>, headers: HeaderMap) -> impl IntoResponse {
    if !auth_admin(&state, &headers, true).await {
        return StofResponse::error(StatusCode::FORBIDDEN, "access denied");
    }

    let count;
    {
        let mut metrics = state.metrics.lock().await;
        count = registry_downloads_count(&mut metrics, &path);
    }

    let stof = format!("int count: {}", count);
    StofResponse::stof(StatusCode::OK, &stof)
}
