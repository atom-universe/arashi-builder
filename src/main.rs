mod cli;
mod middleware;
mod utils;

use crate::middleware::dependency_analysis::DependencyAnalysis;
use crate::middleware::static_file::StaticFiles;
use crate::middleware::tsx_transform::TypescriptTransform;
use clap::Parser;
use cli::{Cli, Commands};
use utils::fs;

async fn start_server(url: &str) {
    let mut app = tide::new();
    let working_dir = fs::get_current_dir().unwrap().to_string_lossy().to_string();

    app.with(DependencyAnalysis::new(working_dir.clone()));
    app.with(StaticFiles::new(working_dir.clone()));
    app.with(TypescriptTransform::new(working_dir.clone()));

    println!(
        "========== 启动 ==========\n URL: http://{} \n========== RUST ==========",
        url
    );
    app.listen(url).await.unwrap();
}

#[async_std::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Dev { port } => {
            start_server(&port).await;
        }
    }
}
