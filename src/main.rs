mod cli;
mod middleware;
mod utils;

use crate::middleware::css_transform::CssTransform;
use crate::middleware::dependency_analysis::DependencyAnalysis;
use crate::middleware::static_file::StaticFiles;
use crate::middleware::tsx_transform::TypescriptTransform;
use clap::Parser;
use cli::{Cli, Commands};
use utils::fs;
// use utils::prebuild;

async fn start_server(url: &str) {
    let mut app = tide::new();
    let working_dir = fs::get_current_dir().unwrap().to_string_lossy().to_string();

    // 1. 按需处理 node_modules 和依赖分析
    app.with(DependencyAnalysis::new(working_dir.clone()).await);
    // 2. TypeScript 转换
    app.with(TypescriptTransform::new(working_dir.clone()));
    // 3. CSS 转换
    app.with(CssTransform::new(working_dir.clone()));
    // 4. 静态文件服务
    app.with(StaticFiles::new(working_dir.clone()));

    println!(
        "========== 启动 ==========\n URL: http://{} \n========== RUST ==========",
        url
    );
    // 这里必须要 await 一下，不然 cli 命令之后就结束了不会等待阻塞
    app.listen(url).await;
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
