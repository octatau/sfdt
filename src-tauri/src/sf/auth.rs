use axum::{extract::Query, extract::State, response::IntoResponse, routing::get, Router};
use lazy_static::lazy_static;
use oauth2::basic::BasicClient;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, CsrfToken, PkceCodeChallenge, PkceCodeVerifier,
    RedirectUrl, TokenResponse, TokenUrl,
};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::oneshot::{channel, Sender};
use tokio::sync::Mutex;

lazy_static! {
    static ref SHUTDOWN_TX: Arc<Mutex<Option<Sender<()>>>> = <_>::default();
}

#[derive(Debug, Clone)]
pub struct OAuthFlow {
    pub csrf_token: CsrfToken,
    pub pkce_challenge: PkceCodeChallenge,
    pub pkce_verifier: String,
    pub client: BasicClient,
    redirect_port: String,
}

impl OAuthFlow {
    pub fn new(base_url: &str, client_id: &str, redirect_port: &str) -> OAuthFlow {
        let auth_url = format!("{base_url}/services/oauth2/authorize");
        let token_url = format!("{base_url}/services/oauth2/token");
        let redirect_url = format!("http://localhost:{redirect_port}/oauth/callback");

        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        let client = BasicClient::new(
            ClientId::new(client_id.to_string()),
            None,
            AuthUrl::new(auth_url).unwrap(),
            Some(TokenUrl::new(token_url).unwrap()),
        )
        .set_redirect_uri(RedirectUrl::new(redirect_url).unwrap());

        OAuthFlow {
            csrf_token: CsrfToken::new_random(),
            pkce_challenge,
            pkce_verifier: PkceCodeVerifier::secret(&pkce_verifier).to_string(),
            client,
            redirect_port: redirect_port.to_string(),
        }
    }

    pub async fn start(&self) {
        let (auth_url, _) = self
            .client
            .authorize_url(|| self.csrf_token.clone())
            .set_pkce_challenge(self.pkce_challenge.clone())
            .url();

        let _ = open::that(auth_url.to_string());

        // start server to handle the oauth callback
        launch_server(self.clone()).await;
    }
}

/** CALLBACK SERVER CONFIGURATION */

#[derive(Deserialize)]
struct OAuthCallbackQuery {
    code: AuthorizationCode,
    state: CsrfToken,
}

async fn launch_server(oauth_flow: OAuthFlow) {
    let (tx, rx) = channel::<()>();
    SHUTDOWN_TX.lock().await.replace(tx);

    let app: Router = Router::new()
        .route("/oauth/callback", get(handle_oauth_callback))
        .with_state(oauth_flow.clone());

    println!("STARTING OAUTH CALLBACK SERVER");
    let server = axum::Server::bind(
        &format!("0.0.0.0:{}", oauth_flow.redirect_port)
            .parse()
            .unwrap(),
    )
    .serve(app.into_make_service());

    let graceful = server.with_graceful_shutdown(async {
        rx.await.ok();
    });

    if let Err(e) = graceful.await {
        eprintln!("server error: {}", e);
    }
}

async fn handle_oauth_callback(
    State(oauth_flow): State<OAuthFlow>,
    query: Query<OAuthCallbackQuery>,
) -> impl IntoResponse {
    if query.state.secret() != oauth_flow.csrf_token.secret() {
        shutdown_server().await;
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
            println!("TOKEN => {:?}", resp.access_token().secret());
            println!("REFRESH TOKEN => {:?}", resp.refresh_token().unwrap().secret());
        },
        Err(err) => {
            println!("ERROR => {:?}", err);
            shutdown_server().await;
            return "unauthorized".to_string();
        }
    }

    shutdown_server().await;
    "authorized".to_string()
}

async fn shutdown_server() {
    if let Some(tx) = SHUTDOWN_TX.lock().await.take() {
        println!("SHUTTING DOWN OAUTH CALLBACK SERVER");
        let _ = tx.send(());
    }
}
