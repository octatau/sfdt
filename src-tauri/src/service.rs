use crate::oauth;
use crate::server;
use crate::sf;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct GlobalState {
    pub client_id: String,
    pub oauth_flow: Arc<Mutex<Option<oauth::OAuthFlow>>>,
    pub callback_server: Arc<Mutex<server::CallbackServer>>,
    pub sf_api: Arc<Mutex<sf::api::RequestHandler>>,
}

impl GlobalState {
    pub async fn init_auth_flow(&self, base_url: &str) {
        let port = self.callback_server.lock().await.get_port();
        let flow = oauth::OAuthFlow::new(base_url, &self.client_id, &port);
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

        let oauth_client = self.oauth_flow.lock().await.clone().unwrap().client;
        let mut base_url = oauth_client.auth_url().url().clone();
        let client_id = oauth_client.client_id().to_string();

        if let Ok(mut path) = base_url.path_segments_mut() {
            path.clear();
        }

        let mut api = self.sf_api.lock().await;
        api.set_config(base_url.to_string(), auth_token);

        match api.get_user_info().await {
            Ok(user_info) => println!("{}", user_info),
            Err(error) => println!("[auth error] {}", error),
        }

        self.cleanup_auth().await;
    }

    async fn cleanup_auth(&self) {
        let _ = self.oauth_flow.lock().await.take();
        self.callback_server.lock().await.stop().await;
    }
}
