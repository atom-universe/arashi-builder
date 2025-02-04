use crate::utils::fs;
use serde_yaml;
use std::collections::HashMap;
use tide::{Next, Request};

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
struct PackageInfo {}

#[async_trait::async_trait]
impl<State: Clone + Send + Sync + 'static> tide::Middleware<State> for DependencyAnalysis {
    async fn handle(&self, req: Request<State>, next: Next<'_, State>) -> tide::Result {
        let mut pkg_name_2_pkg_path = HashMap::<String, String>::new();
        let path_string = std::path::Path::new(&self.working_dir)
            .join("pnpm-lock.yaml")
            .to_string_lossy()
            .to_string();

        let content = fs::read_file_content(&path_string).unwrap().to_string();
        let lock: PnpmLock = serde_yaml::from_str(&content).unwrap();

        for (package_name, _) in lock.packages {
            let module_name = package_name.split('@').collect::<Vec<&str>>()[0];
            let subpath = format!(
                "node_modules/.pnpm/{}/node_modules/{}",
                package_name, module_name
            );

            let pkg_path = std::path::Path::new(&self.working_dir).join(subpath);
            pkg_name_2_pkg_path.insert(package_name, pkg_path.to_string_lossy().to_string());
        }

        let response = next.run(req).await;
        Ok(response)
    }
}
