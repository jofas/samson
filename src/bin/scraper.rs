use serde::{Deserialize, Serialize};

use reqwest::Client;

use futures::stream::{FuturesOrdered, StreamExt};

use chrono::offset::Utc;
use chrono::DateTime;

use tracing::{error, info};

use std::fs::File;
use std::io::{Error as IoError, Write};

#[derive(Debug, Deserialize)]
struct LatestTopics {
    topic_list: TopicList,
}

#[derive(Debug, Deserialize)]
struct TopicList {
    topics: Vec<Topic>,
}

#[derive(Debug, Deserialize)]
struct Topic {
    id: u64,
    title: String,
    created_at: DateTime<Utc>,
    post_stream: Option<PostStream>,
}

#[derive(Debug, Deserialize)]
struct PostStream {
    posts: Vec<Post>,
}

#[derive(Debug, Deserialize)]
struct Post {
    id: u64,
    username: String,
    cooked: String,
    raw: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Question {
    title: String,
    body_raw: String,
    body_cooked: String,
    created: DateTime<Utc>,
    username: String,
}

async fn scrape(url: &str) -> Result<Vec<Question>, Error> {
    let client = Client::new();

    let mut questions: Vec<Question> = Vec::new();
    let mut page = 0;

    loop {
        let latest_topics = client
            .get(format!("{url}/latest.json?order=created&page={page}"))
            .send()
            .await?
            .text()
            .await?;

        let latest_topics: LatestTopics = serde_json::from_str(&latest_topics).map_err(|e| {
            error!(latest_topics);
            e
        })?;

        let topics = latest_topics.topic_list.topics;

        if topics.is_empty() {
            break;
        }

        for (i, topic) in topics.into_iter().enumerate() {
            info!(forum = url, page = page, topic = i);

            let topic = client
                .get(format!("{}/t/{}.json", url, topic.id))
                .send()
                .await?
                .text()
                .await?;

            let topic: Topic = serde_json::from_str(&topic).map_err(|e| {
                error!(topic);
                e
            })?;

            let post_id = topic.post_stream.ok_or(Error::None)?.posts[0].id;

            let post = client
                .get(format!("{url}/posts/{post_id}.json"))
                .send()
                .await?
                .text()
                .await?;

            let post: Post = serde_json::from_str(&post).map_err(|e| {
                error!(post);
                e
            })?;

            let q = Question {
                title: topic.title,
                created: topic.created_at,
                username: post.username,
                body_cooked: post.cooked,
                body_raw: post.raw.ok_or(Error::None)?,
            };

            questions.push(q);
        }

        page += 1;
    }

    Ok(questions)
}

#[derive(Debug)]
enum Error {
    None,
    Reqwest(reqwest::Error),
    Json(serde_json::Error),
    IO(IoError),
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Self::Reqwest(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

impl From<IoError> for Error {
    fn from(e: IoError) -> Self {
        Self::IO(e)
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    let u = "https://users.rust-lang.org";
    let i = "https://internals.rust-lang.org";

    let mut requests = FuturesOrdered::from_iter([scrape(u), scrape(i)]);

    let u = requests.next().await.ok_or(Error::None)??;
    let i = requests.next().await.ok_or(Error::None)??;

    let u = serde_json::to_string_pretty(&u)?;
    let i = serde_json::to_string_pretty(&i)?;

    File::create("urlo.json")?.write_all(u.as_bytes())?;
    File::create("irlo.json")?.write_all(i.as_bytes())?;

    Ok(())
}
