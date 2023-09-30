#[derive(Debug)]
pub struct RequestConfig {
    pub base_url: String,
    pub auth_token: String,
}

#[derive(Debug)]
pub struct RequestHandler {
    config: Option<RequestConfig>,
}

impl RequestHandler {
    pub fn init() -> RequestHandler {
        RequestHandler { config: None }
    }

    pub fn set_config(&mut self, base_url: String, auth_token: String) {
        self.config = Some(RequestConfig {
            base_url,
            auth_token,
        });
    }

    pub async fn get_user_info(&self) -> Result<String, String> {
        self.get("/services/oauth2/userinfo").await
    }

    async fn get(&self, path: &str) -> Result<String, String> {
        match &self.config {
            Some(config) => {
                let url = format!("{}{}", config.base_url, path);
                let v = reqwest::Client::new()
                    .get(url)
                    .bearer_auth(&config.auth_token)
                    .send()
                    .await
                    .unwrap()
                    .json::<serde_json::Value>()
                    .await
                    .unwrap();
                Ok(v.to_string())
            }
            None => {
                println!("[api] no configuration provided");
                Err("no configuration set for sf api handler".to_string())
            }
        }
    }
}
