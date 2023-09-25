use crate::oauth;
use axum::{extract::Query, extract::State, response::IntoResponse, routing::get, Router};
use oauth2::{AuthorizationCode, CsrfToken, PkceCodeVerifier, TokenResponse};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::oneshot::{channel, Sender};
use tokio::sync::Mutex;

#[derive(Clone, Debug)]
enum ServerStatus {
    Up(Arc<Mutex<Option<Sender<()>>>>),
    Down,
}

#[derive(Clone, Debug)]
pub struct CallbackServer {
    port: String,
    status: ServerStatus,
}

impl CallbackServer {
    pub fn init(port: String) -> CallbackServer {
        CallbackServer {
            port,
            status: ServerStatus::Down,
        }
    }

    pub async fn start(&mut self, oauth_flow: oauth::OAuthFlow) {
        match self.status {
            ServerStatus::Up(_) => println!("[callback server] server already running"),
            ServerStatus::Down => {
                let (tx, rx) = channel::<()>();

                self.status = ServerStatus::Up(Arc::new(Mutex::new(Some(tx))));

                println!("{:?}", self);

                let app: Router = Router::new()
                    .route("/oauth/callback", get(handle_oauth_callback))
                    .with_state(oauth_flow);

                println!("[callback server] starting oauth callback server");
                let server = axum::Server::bind(&format!("0.0.0.0:{}", self.port).parse().unwrap())
                    .serve(app.into_make_service());

                let graceful = server.with_graceful_shutdown(async {
                    rx.await.ok();
                });

                graceful.await.unwrap();
            }
        }
    }

    pub async fn stop(&self) {
        println!("{:?}", self);
        match &self.status {
            ServerStatus::Up(shutdown_tx) => {
                if let Some(tx) = shutdown_tx.lock().await.take() {
                    println!("[callback server] shutting down server");
                    let _ = tx.send(());
                }
            }
            ServerStatus::Down => println!("[callback server] server already down"),
        }
    }
}

#[derive(Deserialize)]
struct OAuthCallbackQuery {
    code: AuthorizationCode,
    state: CsrfToken,
}

async fn handle_oauth_callback(
    State(oauth_flow): State<oauth::OAuthFlow>,
    query: Query<OAuthCallbackQuery>,
) -> impl IntoResponse {
    if query.state.secret() != oauth_flow.csrf_token.secret() {
        return "unauthorized".to_string();
    }

    let token_response = oauth_flow
        .client
        .exchange_code(query.code.clone())
        .set_pkce_verifier(PkceCodeVerifier::new(oauth_flow.pkce_verifier.clone()))
        .request_async(oauth2::reqwest::async_http_client)
        .await;

    match token_response {
        Ok(resp) => {
            println!("[auth token] {}", resp.access_token().secret().to_string());
            println!(
                "[refresh token] {}",
                resp.refresh_token().unwrap().secret().to_string()
            );
        }
        Err(err) => {
            println!("ERROR => {:?}", err);
            return "unauthorized".to_string();
        }
    }

    "authorized".to_string()
}
