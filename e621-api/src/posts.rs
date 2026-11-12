use reqwest_middleware::ClientWithMiddleware;
use serde::{Deserialize, Serialize};


/// The entire e621 API JSON response within the "post" field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
	pub id: i64,
	pub created_at: String,
	pub updated_at: String,
	pub change_seq: i64,
	pub files: Files,
	pub uploader_id: i64,
	pub uploader_name: String,
	pub approver_id: Option<i64>,
	pub stats: Stats,
	pub flags: Flags,
	pub has: Has,
	pub relationships: Relationships,
	pub pools: Vec<i64>,
	pub rating: String,
	pub locked_tags: Vec<String>,
	pub sources: Vec<String>,
	pub description: String,
	pub tags: Tags,
}

/// File subfield of JSON response which described the file's dimensions, size, hash, extension and
/// URL on the CDN
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Files {
	pub meta: Meta,
	pub original: FileDescriptor,
	pub preview: FileDescriptor,
	pub sample: FileDescriptor,
	pub video: Option<Video>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meta {
	pub md5: String,
	pub ext: String,
	pub size: i64,
	pub duration: Option<f64>,
	pub has_sample: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDescriptor {
	pub width: i64,
	pub height: i64,
	pub url: Option<String>,
	pub jpg: Option<String>,
	pub webp: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Video {
	pub has: bool,
	pub original: Variant,
	pub variants: Sample,
	pub samples: Sample,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variant {
	pub fps: f64,
	pub codec: String,
	pub size: i64,
	pub width: i64,
	pub height: i64,
	pub url: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sample {
	pub mp4: Option<Variant>,
	pub _480p: Option<Variant>,
	pub _720p: Option<Variant>,
}

/// Score subfield which has the count of upvotes, downvotes and total score of the post
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stats {
	score: Score,
	fav_count: i64,
	is_favorited: bool,
	vote: i64,
	comment_count: i64,
	hotness: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Score {
	pub up: i64,
	pub down: i64,
	pub total: i64,
}

/// Newer tag subfield of most modern posts. It holds each tag in its corresponding category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tags {
	pub general: Vec<String>,
	pub artist: Vec<String>,
	pub contributor: Vec<String>,
	pub copyright: Vec<String>,
	pub character: Vec<String>,
	pub species: Vec<String>,
	pub invalid: Vec<String>,
	pub meta: Vec<String>,
	pub lore: Vec<String>,
}

pub struct TagsIntoIterator {
	tags: Tags,
	index: usize,
}

impl Iterator for TagsIntoIterator {
	type Item = Vec<String>;
	fn next(&mut self) -> Option<Self::Item> {
		let result = match self.index {
			0 => &self.tags.general,
			1 => &self.tags.artist,
			2 => &self.tags.contributor,
			3 => &self.tags.copyright,
			4 => &self.tags.character,
			5 => &self.tags.species,
			6 => &self.tags.invalid,
			7 => &self.tags.meta,
			8 => &self.tags.lore,
			_ => return None,
		};
		self.index += 1;
		Some(result.clone())
	}
}

impl IntoIterator for Tags {
	type Item = Vec<String>;
	type IntoIter = TagsIntoIterator;
	fn into_iter(self) -> Self::IntoIter {
		TagsIntoIterator {
			tags: self,
			index: 0,
		}
	}
}

/// Flags subfield which tells if a post has been flagged, is pending approval, has any kind of lock
/// or has been deleted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Flags {
	pub pending: bool,
	pub flagged: bool,
	pub note_locked: bool,
	pub status_locked: bool,
	pub rating_locked: bool,
	pub deleted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Has {
	parent: bool,
	children: bool,
	active_children: bool,
	notes: bool,
	sample: bool,
}

/// Relationship subfield which tells if a post has a parent or children post.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationships {
	pub parent_id: Option<i64>,
	pub children: Vec<i64>,
}

pub async fn get_post(id: i64, client: ClientWithMiddleware) -> Post {
	let post_url = format!("https://e621.net/posts/{id}.json?v2=true&mode=extended");

	let request = client.get(post_url).build().expect("Failed to build request");
	let response = client.execute(request)
		.await.expect("Request GET /posts/{id}.json failed")
		.json::<Post>()
		.await.expect("Failed to convert response to Root object");
	response
}
