pub fn get_base_url(custom_domain: Option<&str>, is_sandbox: bool) -> String {
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

    format!("https://{subdomain}.salesforce.com")
}
