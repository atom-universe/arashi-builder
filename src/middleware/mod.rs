use crate::utils::fs;
use serde_yaml;
use std::collections::HashMap;
use tide::{Next, Request};

#[derive(Debug, Clone)]
pub struct Logger;

#[async_trait::async_trait]
impl<State: Clone + Send + Sync + 'static> tide::Middleware<State> for Logger {
    async fn handle(&self, req: Request<State>, next: Next<'_, State>) -> tide::Result {
        println!("测试");
        let response = next.run(req).await;
        Ok(response)
    }
}

#[derive(Debug, Clone)]
pub struct DependencyAnalysis {
    pub working_dir: String,
}

impl DependencyAnalysis {
    pub fn new(working_dir: String) -> Self {
        DependencyAnalysis { working_dir }
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct PnpmLock {
    packages: HashMap<String, PackageInfo>,
}

#[derive(Debug, serde::Deserialize)]
struct PackageInfo {
    resolution: Option<Resolution>,
    #[serde(default)]
    dependencies: Option<HashMap<String, String>>,
    #[serde(rename = "peerDependencies")]
    peer_dependencies: Option<HashMap<String, String>>,
}

#[derive(Debug, serde::Deserialize)]
struct Resolution {
    integrity: String,
}

#[async_trait::async_trait]
impl<State: Clone + Send + Sync + 'static> tide::Middleware<State> for DependencyAnalysis {
    async fn handle(&self, req: Request<State>, next: Next<'_, State>) -> tide::Result {
        println!("也被调用了！{}", &self.working_dir);
        let mut pkg_name_2_pkg_path = HashMap::<String, String>::new();
        let path_string = std::path::Path::new(&self.working_dir)
            .join("pnpm-lock.yaml")
            .to_string_lossy()
            .to_string();

        let content = fs::read_file_content(&path_string).unwrap().to_string();
        let lock: PnpmLock = serde_yaml::from_str(&content).unwrap();

        // 打印所有包名
        for (package_name, _) in lock.packages {
            println!("Package: {}", package_name);
            let module_name = package_name.split('@').collect::<Vec<&str>>()[0];
            println!("module_name: {}", module_name);
            let subpath = format!(
                "node_modules/.pnpm/{}/node_modules/{}",
                package_name, module_name
            );

            let pkg_path = std::path::Path::new(&self.working_dir).join(subpath);
            pkg_name_2_pkg_path.insert(package_name, pkg_path.to_string_lossy().to_string());
        }

        println!("{:?}", pkg_name_2_pkg_path);

        let response = next.run(req).await;
        Ok(response)
    }
}
