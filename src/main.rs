use clap::{Parser, Subcommand};
use tide::utils::After;
use tide::{Next, Request};

#[derive(Debug, Clone)]
struct Logger;

#[async_trait::async_trait]
impl<State: Clone + Send + Sync + 'static> tide::Middleware<State> for Logger {
    async fn handle(&self, req: Request<State>, next: Next<'_, State>) -> tide::Result {
        let response = next.run(req).await;
        println!("被调用了");
        Ok(response)
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 启动开发服务器
    Dev {
        #[arg(short, long, default_value = "127.0.0.1:8080")]
        port: String,
    },
}

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
