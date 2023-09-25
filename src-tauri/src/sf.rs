mod oauth_handler;

use crate::oauth;

#[derive(Debug, Clone)]
pub struct Service {
    pub client_id: String,
    pub oauth_port: String,
    pub callback_server: oauth_handler::CallbackServer,
}

impl Service {
    pub fn init(client_id: &str, oauth_port: &str) -> Service {
        Service {
            client_id: client_id.to_string(),
            oauth_port: oauth_port.to_string(),
            callback_server: oauth_handler::CallbackServer::init(oauth_port.to_string()),
        }
    }

    pub async fn authorize_org(&mut self, custom_domain: Option<&str>, is_sandbox: bool) {
        let subdomain: String;

        match custom_domain {
            Some(domain) => {
                let qualifier = if is_sandbox { "sandbox.my" } else { "my" };
                subdomain = format!("{domain}.{qualifier}");
            }
            None => {
                subdomain = (if is_sandbox { "test" } else { "login" }).to_owned();
            }
        }

        let base_url = format!("https://{subdomain}.salesforce.com");

        let flow = oauth::OAuthFlow::new(&base_url, &self.client_id, &self.oauth_port);
        flow.start();

        self.callback_server.start(flow).await;
    }
}
