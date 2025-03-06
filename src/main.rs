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

mod run;
mod response;
mod users;
mod registry;

mod config;
use config::load_config;

mod server;
use server::serve;

use clap::Parser;
use colored::Colorize;


#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long, value_name = "STOF_FILE")]
    config: Option<String>,
}


#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let config = load_config(cli.config);
    match config {
        Ok(config) => {
            serve(config).await;
        },
        Err(error) => {
            println!("{}: {}", "ConfigError".red(), error.dimmed());
        }
    }
}
