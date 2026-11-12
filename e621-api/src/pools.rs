use reqwest_middleware::ClientWithMiddleware;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pool {
	pub id: i64,
	pub name: String,
	pub updated_at: String,
	pub creator_id: i64,
	pub description: String,
	pub is_active: bool,
	pub category: PoolCategory,
	pub post_ids: Vec<i64>,
	pub created_at: String,
	pub creator_name: String,
	pub post_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PoolCategory {
	collection, series, UNINIT
}

pub async fn get_pool(id: i64, client: ClientWithMiddleware) -> Pool {
	let post_url = format!("https://e621.net/pools/{id}.json?v2=true&mode=extended");

	let request = client.get(post_url).build().expect("Failed to build request");
	let response = client.execute(request)
		.await.expect("Request GET /pools/{id}.json failed")
		.json::<Pool>()
		.await.expect("Failed to convert response to Root object");
	response
}

impl Pool {
	pub fn new() -> Pool {
		Pool {
			id: 0,
			name: "UNINITIALIZED".to_string(),
			updated_at: "UNINITIALIZED".to_string(),
			creator_id: 0,
			description: "UNINITIALIZED".to_string(),
			is_active: false,
			category: PoolCategory::UNINIT,
			post_ids: vec![],
			created_at: "UNINITIALIZED".to_string(),
			creator_name: "UNINITIALIZED".to_string(),
			post_count: 0,
		}
	}
}