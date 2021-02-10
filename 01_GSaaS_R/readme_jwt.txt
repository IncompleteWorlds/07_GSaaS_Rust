JSON WEB TOOLKIT

pub fn get_jwt(api_key: &str, api_secret: &str) -> Result<String, Error> {
    let mut body = HashMap::new();
    body.insert("apiKey", api_key);
    body.insert("apiSecret", api_secret);
    let jwt_path = format!("{}/developer/sign-in", BASE_URL);
    let result = post(&jwt_path, &body, "")?;
    let json: Result<SignInResponse, Error> = serde_json::from_str(&result)
        .map_err(|e| format_err!("could not parse json, reason: {}", e));
    Ok(json.unwrap().token)
}


pub struct Config {
    api_key: String,

    api_secret: String,
}

