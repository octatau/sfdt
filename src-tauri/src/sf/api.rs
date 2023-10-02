const DEFAULT_API_VERSION: &str = "58.0";

enum RequestType {
    GET,
    POST,
}

#[derive(Debug)]
pub struct RequestContext {
    pub base_url: String,
    pub auth_token: String,
    pub api_version: String,
}

#[derive(Debug)]
pub struct RequestHandler {
    context: Option<RequestContext>,
    client: reqwest::Client,
}

impl RequestHandler {
    pub fn init() -> RequestHandler {
        RequestHandler {
            context: None,
            client: reqwest::Client::new(),
        }
    }

    pub fn set_config(&mut self, base_url: String, auth_token: String, api_version: Option<&str>) {
        self.context = Some(RequestContext {
            base_url,
            auth_token,
            api_version: api_version.unwrap_or(DEFAULT_API_VERSION).to_string(),
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

    pub async fn get_api_versions(&self) -> Result<serde_json::Value, reqwest::Error> {
        let url = self.build_url("/services/data", None);
        self.request(RequestType::GET, url).await
    }

    pub async fn get_query_results(
        &self,
        query: &str,
    ) -> Result<serde_json::Value, reqwest::Error> {
        let api_version = &self.context.as_ref().unwrap().api_version;
        let url = self.build_url(
            &format!("/services/data/v{api_version}/query"),
            Some(vec![("q", query)]),
        );
        self.request(RequestType::GET, url).await
    }

    fn build_url(&self, path: &str, url_parameters: Option<Vec<(&str, &str)>>) -> String {
        super::url::build_url(
            &self.context.as_ref().unwrap().base_url,
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
        let context = self.context.as_ref().unwrap();

        match req_type {
            RequestType::GET => {
                self.client
                    .get(url)
                    .bearer_auth(&context.auth_token)
                    .send()
                    .await
            }
            RequestType::POST => {
                self.client
                    .post(url)
                    .bearer_auth(&context.auth_token)
                    .send()
                    .await
            }
        }
    }
}
