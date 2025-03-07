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

use std::{net::SocketAddr, sync::Arc, time::Duration};
use axum::{routing::{get, post}, Router};
use colored::Colorize;
use stof::SDoc;
use tokio::sync::Mutex;
use tower_governor::{governor::GovernorConfig, GovernorLayer};
use tower_http::cors::CorsLayer;
use crate::{config::{server_address, server_port}, metrics::{api::{get_downloads_count_handler, get_packages_count_handler, get_server_run_count_handler, get_total_downloads_count_handler}, load_metrics}, registry::{api::{delete_registry_handler, get_registry_handler, publish_registry_handler}, system::SystemRegistry, Registry}, run::run_handler, users::{api::{admin_delete_user_handler, admin_set_user_handler}, load_users}};


/// Server state.
#[derive(Clone)]
pub struct ServerState {
    /// Configuration document.
    pub config: Arc<Mutex<SDoc>>,

    /// Users document.
    pub users: Arc<Mutex<SDoc>>,

    /// Metrics.
    pub metrics: Arc<Mutex<SDoc>>,

    /// Registry.
    pub registry: Arc<Mutex<dyn Registry>>,
}


/// Start the runner server.
pub async fn serve(config: SDoc) {
    // Setup governor configuration - see https://crates.io/crates/tower_governor
    let governor_conf = Arc::new(GovernorConfig::default());
    let governor_limiter = governor_conf.limiter().clone();
    let interval = Duration::from_secs(60);
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(interval);
            governor_limiter.retain_recent();
        }
    });

    let cors = CorsLayer::permissive();
    let address = SocketAddr::from((server_address(&config), server_port(&config)));
    let users = load_users(&config);
    let metrics = load_metrics(&config);
    let registry = SystemRegistry::new(&config);
    let state = ServerState {
        config: Arc::new(Mutex::new(config)),
        users: Arc::new(Mutex::new(users)),
        registry: Arc::new(Mutex::new(registry)),
        metrics: Arc::new(Mutex::new(metrics)),
    };

    let app = Router::new()
        // Registry API
        .route("/registry/{*path}", get(get_registry_handler)
            .put(publish_registry_handler)
            .delete(delete_registry_handler))

        // Run API
        .route("/run", post(run_handler))

        // Admin Users API
        .route("/admin/users", post(admin_set_user_handler)
            .delete(admin_delete_user_handler))

        // Admin Metrics API
        .route("/admin/metrics/run", get(get_server_run_count_handler))
        .route("/admin/metrics/packages", get(get_packages_count_handler))
        .route("/admin/metrics/downloads", get(get_total_downloads_count_handler))
        .route("/admin/metrics/downloads/{*path}", get(get_downloads_count_handler))
        
        .layer(GovernorLayer {
            config: governor_conf
        })
        .layer(cors)
        .with_state(state);
    
    let listener = tokio::net::TcpListener::bind(address)
        .await
        .unwrap();
    
    println!("{} {} {}", "stof-runner".purple(), "listening on".dimmed(), listener.local_addr().unwrap().to_string().bright_cyan().bold());

    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}
