mod cli;
mod middleware;
mod utils;

use clap::Parser;
use cli::{Cli, Commands};
use middleware::{DependencyAnalysis, Logger, StaticFiles};
use utils::fs;

async fn start_server(url: &str) {
    let mut app = tide::new();
    let working_dir = fs::get_current_dir().unwrap().to_string_lossy().to_string();

    app.with(DependencyAnalysis::new(working_dir.clone()));
    app.with(StaticFiles::new(working_dir));

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
