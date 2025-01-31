use tide::{Next, Request};

#[derive(Debug, Clone)]
pub struct Logger;

#[async_trait::async_trait]
impl<State: Clone + Send + Sync + 'static> tide::Middleware<State> for Logger {
    async fn handle(&self, req: Request<State>, next: Next<'_, State>) -> tide::Result {
        let response = next.run(req).await;
        println!("被调用了");
        Ok(response)
    }
}
