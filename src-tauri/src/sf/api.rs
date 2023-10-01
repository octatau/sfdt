enum RequestType {
    GET,
    POST,
}

#[derive(Debug)]
pub struct RequestConfig {
    pub base_url: String,
    pub auth_token: String,
}

#[derive(Debug)]
pub struct RequestHandler {
    config: Option<RequestConfig>,
    client: reqwest::Client,
}

impl RequestHandler {
    pub fn init() -> RequestHandler {
        RequestHandler {
            config: None,
            client: reqwest::Client::new(),
        }
    }

    pub fn set_config(&mut self, base_url: String, auth_token: String) {
        self.config = Some(RequestConfig {
            base_url,
            auth_token,
        });
    }

    pub async fn refresh_auth_token(
        &self,
        client_id: String,
        refresh_token: String,
    ) -> Result<serde_json::Value, reqwest::Error> {
        let params = vec![
            ("grant_type", "refresh_token"),
            ("client_id", &client_id),
            ("refresh_token", &refresh_token),
        ];

        let url = self.build_url("/services/oauth2/token", Some(params));

        self.request(RequestType::POST, url).await
    }

    pub async fn get_user_info(&self) -> Result<serde_json::Value, reqwest::Error> {
        let url = self.build_url("/services/oauth2/userinfo", None);
        self.request(RequestType::GET, url).await
    }

    fn build_url(&self, path: &str, url_parameters: Option<Vec<(&str, &str)>>) -> String {
        super::url::build_url(
            &self.config.as_ref().unwrap().base_url,
            path,
            url_parameters,
        )
    }

    async fn request(
        &self,
        req_type: RequestType,
        req_url: String,
    ) -> Result<serde_json::Value, reqwest::Error> {
        match self.dispatch_request(&req_type, &req_url).await {
            Ok(resp) => match resp.json::<serde_json::Value>().await {
                Ok(json) => Ok(json),
                Err(error) => Err(error),
            },
            Err(error) => Err(error),
        }
    }

    async fn dispatch_request(
        &self,
        req_type: &RequestType,
        url: &str,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let config = self.config.as_ref().unwrap();

        match req_type {
            RequestType::GET => {
                self.client
                    .get(url)
                    .bearer_auth(&config.auth_token)
                    .send()
                    .await
            }
            RequestType::POST => {
                self.client
                    .post(url)
                    .bearer_auth(&config.auth_token)
                    .send()
                    .await
            }
        }
    }
}
