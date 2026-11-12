use base64::{engine::general_purpose, Engine as _};
use std::time::Duration;
use reqwest_middleware::{ClientBuilder};
use reqwest::header;
use reqwest_leaky_bucket::leaky_bucket::RateLimiter;
use crate::profiles;

pub static USER_AGENT: &str = "e621-api library v0.1.0 | https://t.me/HazzyXn";

/// Creates an authentication token with the supplied username
/// and API key.
pub fn create_auth_token(username: &str, api_key: &str) -> String {
	"Basic ".to_owned() + &*general_purpose::STANDARD.encode(format!("{username}:{api_key}"))
}

pub async fn get_profile(token: Option<String>) -> profiles::Profile {
	let limiter = RateLimiter::builder()
		.interval(Duration::from_millis(500))
		.max(10)
		.initial(0)
		.refill(5)
		.build();

	let mut headers = header::HeaderMap::new();
	headers.insert(header::USER_AGENT, header::HeaderValue::from_static(USER_AGENT));
	if token.is_some() {
		headers.insert(header::AUTHORIZATION, token.unwrap().parse().unwrap());
	}

	let client = ClientBuilder::new(
		reqwest::Client::builder()
			.default_headers(headers)
			.build()
			.expect("Failed to build client"),
	)
		.with(reqwest_leaky_bucket::rate_limit_all(limiter))
		.build();

	let request = client.get("https://e621.net/users/me.json").build().expect("Failed to build request");
	let response = client.execute(request)
		.await.expect("Request GET /users/me.json failed")
		.json::<profiles::Profile>()
		.await.expect("Failed to convert response to Profile object");
	response
}