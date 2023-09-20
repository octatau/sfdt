mod auth;

#[derive(Debug)]
pub struct Service {
    pub client_id: String,
    pub oauth_port: String,
}

impl Service {
    pub fn init(client_id: &str, oauth_port: &str) -> Service {
        Service {
            client_id: client_id.to_string(),
            oauth_port: oauth_port.to_string(),
        }
    }

    pub async fn authenticate_org(&self, custom_domain: Option<&str>, is_sandbox: bool) {
        let subdomain: String;

        match custom_domain {
            Some(domain) => {
                let qualifier = if is_sandbox { "sandbox.my" } else { "my" };
                subdomain = format!("{domain}.{qualifier}");
            },
            None => {
                subdomain = (if is_sandbox { "test" } else { "login" }).to_owned();
            },
        }

        let base_url = format!("https://{subdomain}.salesforce.com");

        let flow = auth::OAuthFlow::new(&base_url, &self.client_id, &self.oauth_port);
        flow.start().await;
    }
}