use serde::{de::DeserializeOwned, Deserialize, Serialize};

use reqwest::Client;

use futures::stream::{FuturesUnordered, StreamExt};

use chrono::offset::Utc;
use chrono::DateTime;

use tracing::{error, info};

use tokio::time::sleep;

use regex::Regex;

use once_cell::sync::Lazy;

use std::fs::{create_dir_all, File};
use std::io::{self, Write};
use std::num::ParseIntError;
use std::time::Duration;

static WAIT_SECONDS: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"Please retry again in (?P<s>\d{1,2}) seconds").unwrap());

macro_rules! log_error {
    ($msg:literal) => {
        |e| {
            error!("{}: {e:?}", $msg);
            e
        }
    };
}

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

async fn get<T: DeserializeOwned>(client: &Client, url: &str) -> Result<T, Error> {
    loop {
        let response = client.get(url).send().await?;

        let status = response.status();

        let body = response.text().await?;

        if status.as_u16() == 429 {
            let seconds: u64 = WAIT_SECONDS
                .captures(&body)
                .ok_or(Error::None)
                .map_err(log_error!("no capture found"))?
                .name("s")
                .ok_or(Error::None)
                .map_err(log_error!("no capture name `s` found"))?
                .as_str()
                .parse()?;

            info!("sleeping for {seconds} seconds");

            sleep(Duration::from_secs(seconds)).await;

            continue;
        }

        return Ok(serde_json::from_str(&body)?);
    }
}

async fn scrape(name: &str, url: &str) -> Result<(), Error> {
    create_dir_all("scrape/")?;

    let client = Client::new();

    let mut page = 0;

    loop {
        let mut questions: Vec<Question> = Vec::with_capacity(30);

        let latest_topics: LatestTopics = get(
            &client,
            &format!("{url}/latest.json?order=created&page={page}"),
        )
        .await?;

        let topics = latest_topics.topic_list.topics;

        if topics.is_empty() {
            break;
        }

        for (i, topic) in topics.into_iter().enumerate() {
            info!(forum = url, page = page, topic = i);

            let topic: Topic = get(&client, &format!("{}/t/{}.json", url, topic.id)).await?;

            let post_id = topic
                .post_stream
                .ok_or(Error::None)
                .map_err(log_error!("`post_stream` empty"))?
                .posts[0]
                .id;

            let post: Post = get(&client, &format!("{url}/posts/{post_id}.json")).await?;

            let q = Question {
                title: topic.title,
                created: topic.created_at,
                username: post.username,
                body_cooked: post.cooked,
                body_raw: post
                    .raw
                    .ok_or(Error::None)
                    .map_err(log_error!("`post.raw` field empty"))?,
            };

            questions.push(q);
        }

        let res = serde_json::to_vec_pretty(&questions)?;
        File::create(format!("scrape/{name}-{page}.json"))?.write_all(&res)?;

        page += 1;
    }

    Ok(())
}

#[derive(Debug)]
enum Error {
    None,
    Reqwest(reqwest::Error),
    Json(serde_json::Error),
    IO(io::Error),
    ParseIntError(ParseIntError),
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

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self::IO(e)
    }
}

impl From<ParseIntError> for Error {
    fn from(e: ParseIntError) -> Self {
        Self::ParseIntError(e)
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    let u = "https://users.rust-lang.org";
    let i = "https://internals.rust-lang.org";

    let mut requests = FuturesUnordered::from_iter([scrape("urlo", u), scrape("irlo", i)]);

    while let Some(r) = requests.next().await {
        r?;
    }

    Ok(())
}
