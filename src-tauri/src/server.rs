use crate::{oauth, service};
use axum::{extract::Query, extract::State, response::IntoResponse, routing::get, Router};
use oauth2::{AuthorizationCode, CsrfToken, PkceCodeVerifier, TokenResponse};
use serde::Deserialize;
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::oneshot::{channel, Sender};
use tokio::sync::Mutex;

#[derive(Clone, Debug)]
enum ServerStatus {
    Up(Arc<Mutex<Option<Sender<()>>>>),
    Down,
}

#[derive(Clone, Debug)]
pub struct CallbackServer {
    app_handle: tauri::AppHandle,
    port: String,
    status: ServerStatus,
}

impl CallbackServer {
    pub fn init(app_handle: tauri::AppHandle, port: String) -> CallbackServer {
        CallbackServer {
            app_handle,
            port,
            status: ServerStatus::Down,
        }
    }

    pub async fn start(&mut self) {
        match self.status {
            ServerStatus::Up(_) => println!("[callback server] server already running"),
            ServerStatus::Down => {
                let (tx, rx) = channel::<()>();

                self.status = ServerStatus::Up(Arc::new(Mutex::new(Some(tx))));

                let app: Router = Router::new()
                    .route("/oauth/callback", get(handle_oauth_callback))
                    .with_state(self.app_handle.clone());

                println!("[callback server] starting oauth callback server");
                let server = axum::Server::bind(&format!("0.0.0.0:{}", self.port).parse().unwrap())
                    .serve(app.into_make_service());

                let graceful = server.with_graceful_shutdown(async {
                    rx.await.ok();
                });

                tokio::spawn(async move {
                    graceful.await.unwrap();
                });
            }
        }
    }

    pub async fn stop(&self) {
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
    State(app_handle): State<tauri::AppHandle>,
    query: Query<OAuthCallbackQuery>,
) -> impl IntoResponse {
    let app_state: tauri::State<'_, service::GlobalState> = app_handle.state();
    let oauth_flow_lock = app_state.oauth_flow.lock().await;
    let oauth_flow = oauth_flow_lock.as_ref().unwrap();

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
            std::mem::drop(oauth_flow_lock);
            app_state
                .consume_auth_token(
                    resp.access_token().secret().to_string(),
                    resp.refresh_token().unwrap().secret().to_string(),
                )
                .await;
        }
        Err(err) => {
            println!("ERROR => {:?}", err);
            return "unauthorized".to_string();
        }
    }

    "authorized".to_string()
}
