mod cli;
mod middleware;

use clap::Parser;
use cli::{Cli, Commands};
use middleware::Logger;

async fn start_server(url: &str) {
    let mut app = tide::new();
    app.with(Logger);
    app.at("/").get(|_| async { Ok("Hello, Tide!") });
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
