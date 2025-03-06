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

use std::{collections::BTreeMap, sync::Arc, time::Duration};
use axum::{extract::{Query, State}, http::{header::CONTENT_TYPE, HeaderMap, StatusCode}, response::IntoResponse};
use bytes::Bytes;
use stof::{SDoc, SVal};
use stof_http::HTTPLibrary;
use tokio::time::timeout;
use crate::{config::{opaque_errors, registry_path, run_enabled, run_timeout}, registry::pkg::RPKG, response::StofResponse, server::ServerState, users::auth::auth_exec};
mod sandbox_fs;
use sandbox_fs::TmpFileSystemLibrary;


/// Run API endpoint handler.
pub(crate) async fn run_handler(State(state): State<ServerState>, Query(query): Query<BTreeMap<String, String>>, headers: HeaderMap, body: Bytes) -> impl IntoResponse {
    if !auth_exec(&state, &headers).await {
        return StofResponse::error(StatusCode::FORBIDDEN, "access denied");
    }

    let opaque_stof_errors;
    let run_time;
    let registry;
    {
        let mut config = state.config.lock().await;
        if !run_enabled(&config) {
            return StofResponse::error(StatusCode::NOT_IMPLEMENTED, "runner is not available");
        }
        opaque_stof_errors = opaque_errors(&config);
        run_time = run_timeout(&mut config);
        registry = registry_path(&config);
    }

    let mut content_type = String::from("stof");
    if let Some(ctype) = headers.get(CONTENT_TYPE) {
        content_type = ctype.to_str().unwrap().to_owned();
    }

    let mut export_format = String::from("bstof");
    if let Some(format) = query.get("export") {
        export_format = format.clone();
    }

    run_stof(&content_type, run_time, body, opaque_stof_errors, &export_format, &registry).await
}


/// Run some Stof.
///
/// content_type: content type for the incoming message body.
/// time: timeout for running this Stof.
/// body: bytes of the incoming Stof document.
/// opaque_errors: true if specific error information should be hidden from the response.
/// export_format: default is "bstof", but the resulting document gets exported to this format for the response.
async fn run_stof(content_type: &str, time: Duration, mut body: Bytes, opaque_errors: bool, export_format: &str, registry_path: &str) -> StofResponse {
    let result = timeout(time, async move {
        let mut doc = SDoc::default();
        initialize_document(&mut doc, registry_path).await;
        let res = doc.header_import("main", &content_type, &content_type, &mut body, "");
        match res {
            Ok(_) => {
                // Execute the main root as a task
                if let Some(main) = doc.graph.main_root() {
                    if let Some(lib) = doc.libraries.get("Object") {
                        let res = lib.call("main", &mut doc, "exec", &mut vec![SVal::Object(main)]);
                        match res {
                            Ok(_) => {
                                // Nothing to do here...
                            },
                            Err(res) => {
                                if !opaque_errors {
                                    return StofResponse::error(StatusCode::BAD_REQUEST, &res.to_string(&doc.graph));
                                }
                                return StofResponse::error(StatusCode::BAD_REQUEST, "error executing document");
                            },
                        }
                    }
                }

                // Run the remote functions in this document
                let res = doc.run(None, Some("remote".into()));
                match res {
                    Ok(_) => {
                        // Nothing to do here...
                    },
                    Err(res) => {
                        if !opaque_errors {
                            return StofResponse::error(StatusCode::BAD_REQUEST, &res);
                        }
                        return StofResponse::error(StatusCode::BAD_REQUEST, "error running document");
                    },
                }
            },
            Err(error) => {
                if !opaque_errors {
                    return StofResponse::error(StatusCode::BAD_REQUEST, &error.to_string(&doc.graph));
                }
                return StofResponse::error(StatusCode::BAD_REQUEST, "error parsing document");
            },
        }

        if export_format != "bstof" {
            if let Ok(text) = doc.export_string("main", export_format, None) {
                if let Some(format) = doc.formats.get(export_format) {
                    let mut headers = HeaderMap::new();
                    headers.insert(CONTENT_TYPE, format.content_type().parse().unwrap());
                    return StofResponse {
                        headers,
                        status: StatusCode::OK,
                        str_body: text,
                        bytes_body: None,
                    };
                }
            }
        }

        if let Ok(bytes) = doc.export_bytes("main", "bstof", None) {
            return StofResponse::bstof(StatusCode::OK, bytes);
        }
        StofResponse::error(StatusCode::INTERNAL_SERVER_ERROR, "error exporting document")
    }).await;

    match result {
        Ok(res) => {
            res
        },
        Err(_) => {
            StofResponse::error(StatusCode::REQUEST_TIMEOUT, "timeout while running document")
        }
    }
} 


/// Initialize document.
/// Load additional libraries, etc.
async fn initialize_document(doc: &mut SDoc, registry_path: &str) {
    // Replace the fs library with one that can only access the TMP directory
    doc.load_lib(Arc::new(TmpFileSystemLibrary::default()));

    // Add HTTP library
    doc.load_lib(Arc::new(HTTPLibrary::default()));

    // Add the Registry PKG format in place of the normal PKG format
    // This enables users to load packages from this registry using the familiar "import pkg '@hello/hello'" format
    doc.load_format(Arc::new(RPKG::new(registry_path)));
}
