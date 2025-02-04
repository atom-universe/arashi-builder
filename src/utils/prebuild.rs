use async_std::path::{Path, PathBuf};
use std::collections::{HashMap, HashSet};
use std::process::Command;
use tide::Result;

#[derive(Debug)]
pub struct DepCache {
    cache_dir: PathBuf,
    metadata: HashMap<String, String>, // 包名 -> 预构建文件路径
    building: HashSet<String>,         // 正在构建的包
}

impl DepCache {
    pub async fn new(root_dir: &Path) -> Self {
        let cache_dir = root_dir.join("node_modules/.arashi");
        async_std::fs::create_dir_all(&cache_dir).await.unwrap();

        let deps_dir = cache_dir.join("deps");
        async_std::fs::create_dir_all(&deps_dir).await.unwrap();

        Self {
            cache_dir,
            metadata: HashMap::new(),
            building: HashSet::new(),
        }
    }

    pub async fn get_or_build(&mut self, pkg_name: &str, pkg_path: &Path) -> Result<PathBuf> {
        // 如果已经有缓存，直接返回
        if let Some(path) = self.get_cached_path(pkg_name) {
            return Ok(path);
        }

        // 如果正在构建，等待构建完成
        if self.building.contains(pkg_name) {
            while self.building.contains(pkg_name) {
                async_std::task::sleep(std::time::Duration::from_millis(50)).await;
            }
            return Ok(self.get_cached_path(pkg_name).unwrap());
        }

        // 开始构建
        self.building.insert(pkg_name.to_string());
        self.build_dep(pkg_name, pkg_path).await;
        self.building.remove(pkg_name);

        Ok(self.get_cached_path(pkg_name).unwrap())
    }

    pub async fn build_dep(&mut self, pkg_name: &str, pkg_path: &Path) {
        let outdir = self.cache_dir.join("deps");
        async_std::fs::create_dir_all(&outdir).await.unwrap();

        let pkg_path = pkg_path.to_owned();
        let outdir = outdir.to_owned();
        let pkg_name_clone = pkg_name.to_owned(); // 克隆用于 spawn_blocking

        async_std::task::spawn_blocking(move || {
            Command::new("npx")
                .arg("esbuild")
                .arg(&pkg_path)
                .arg("--bundle")
                .arg("--format=esm")
                .arg("--platform=browser")
                .arg("--target=es2020")
                .arg(format!(
                    "--outfile={}/{}.js",
                    outdir.display(),
                    pkg_name_clone
                ))
                .status()
        })
        .await
        .unwrap();

        self.metadata
            .insert(pkg_name.to_string(), format!("deps/{}.js", pkg_name));
    }

    pub fn get_cached_path(&self, pkg_name: &str) -> Option<PathBuf> {
        self.metadata.get(pkg_name).map(|p| self.cache_dir.join(p))
    }
}
