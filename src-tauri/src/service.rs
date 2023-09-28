use crate::oauth;
use crate::server;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct GlobalState {
    pub client_id: String,
    pub oauth_flow: Arc<Mutex<Option<oauth::OAuthFlow>>>,
    pub callback_server: Arc<Mutex<server::CallbackServer>>,
}

impl GlobalState {
    pub async fn init_auth_flow(&self, base_url: &str) {
        let flow = oauth::OAuthFlow::new(base_url, &self.client_id, "3425");
        self.oauth_flow.lock().await.replace(flow);
    }

    pub async fn launch_auth(&self) {
        match self.oauth_flow.lock().await.as_mut() {
            Some(flow) => {
                self.callback_server.lock().await.start().await;
                flow.start();
            }
            None => println!("[oauth flow] initialize auth flow before attempting to start"),
        }
    }

    pub async fn consume_auth_token(&self, auth_token: String, refresh_token: String) {
        println!("[auth token] {}", auth_token);
        println!("[refresh token] {}", refresh_token);
        self.cleanup_auth().await;
    }

    async fn cleanup_auth(&self) {
        let _ = self.oauth_flow.lock().await.take();
        self.callback_server.lock().await.stop().await;
    }
}
