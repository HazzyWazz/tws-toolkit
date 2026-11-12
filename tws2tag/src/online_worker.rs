use std::time::Duration;
use reqwest::header;
use reqwest::header::HeaderMap;
use reqwest_leaky_bucket::leaky_bucket::RateLimiter;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};

pub fn create_global_limiter(rate: u64) -> RateLimiter {
	RateLimiter::builder()
		.interval(Duration::from_millis(rate))
		.max(10)
		.initial(0)
		.refill(5)
		.build()
}

pub fn create_global_client(headers: HeaderMap, limiter: RateLimiter, ) -> ClientWithMiddleware {
	let client = ClientBuilder::new(
		reqwest::Client::builder()
			.default_headers(headers)
			.build().expect("Cannot build client")
	)
		.with(reqwest_leaky_bucket::rate_limit_all(limiter))
		.build();
	
	client
}

pub fn create_global_headers(user_agent: &str, token: Option<&str>) -> HeaderMap {
	let mut headers = HeaderMap::new();
	headers.insert(header::USER_AGENT, header::HeaderValue::from_str(user_agent).expect("User Agent not valid"));
	headers.insert(header::IF_MODIFIED_SINCE, header::HeaderValue::from_static("Thu, 01 Jan 1970 00:00:00 GMT"));
	if token.is_some() {
		let mut auth_value = header::HeaderValue::from_str(token.unwrap()).unwrap();
		auth_value.set_sensitive(true);
		headers.insert(header::AUTHORIZATION, auth_value);
	}
	
	headers
}