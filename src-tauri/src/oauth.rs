use oauth2::basic::BasicClient;
use oauth2::{
    AuthUrl, ClientId, CsrfToken, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, TokenUrl,
};

#[derive(Debug, Clone)]
pub struct OAuthFlow {
    pub csrf_token: CsrfToken,
    pub pkce_challenge: PkceCodeChallenge,
    pub pkce_verifier: String,
    pub client: BasicClient,
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
        }
    }

    pub fn start(&self) {
        let (auth_url, _) = self
            .client
            .authorize_url(|| self.csrf_token.clone())
            .set_pkce_challenge(self.pkce_challenge.clone())
            .url();

        let _ = open::that(auth_url.to_string());
    }
}
