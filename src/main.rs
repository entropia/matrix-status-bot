use reqwest::{Client, header};
use serde::Deserialize;
use serde_json::json;
use std::cmp::PartialEq;
use std::env;
use std::fmt::Debug;

const APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);
const CLIENT_API: &str = "_matrix/client/v3";
#[derive(Deserialize, Debug, PartialEq)]
enum Status {
    Open,
    Closed,
}

struct Config {
    matrix_auth_token: String,
    matrix_homeserver_address: String,
    matrix_channel_id: String,
    spaceapi_endpoint: String,
    matrix_topic_open: String,
    matrix_topic_closed: String,
}

#[derive(Deserialize)]
struct ApiResponse {
    state: ApiState,
}

#[derive(Deserialize)]
struct ApiState {
    open: bool,
}

#[derive(Deserialize)]
struct MatrixResponse {
    topic: String,
}

async fn get_api_status(config: &Config) -> Status {
    let response = reqwest::get(config.spaceapi_endpoint.clone())
        .await
        .unwrap()
        .json::<ApiResponse>()
        .await
        .unwrap();
    if let true = response.state.open {
        Status::Open
    } else {
        Status::Closed
    }
}

async fn get_channel_topic(config: &Config, http_client: &Client) -> String {
    let res = http_client.get(
        config.matrix_homeserver_address.clone()
            + "/"
            + CLIENT_API
            + "/rooms/"
            + config.matrix_channel_id.as_str()
            + "/state"
            + "/m.room.topic",
    );
    res.send()
        .await
        .unwrap()
        .json::<MatrixResponse>()
        .await
        .unwrap()
        .topic
}

async fn set_channel_topic(topic: String, http_client: &Client, config: &Config) {
    http_client
        .put(
            config.matrix_homeserver_address.clone()
                + "/"
                + CLIENT_API
                + "/rooms/"
                + config.matrix_channel_id.as_str()
                + "/state"
                + "/m.room.topic",
        )
        .json(&json!({"topic": &topic}))
        .send()
        .await
        .unwrap();
}

async fn read_config() -> Config {
    Config {
        matrix_auth_token: env::var("MATRIX_AUTH_TOKEN").unwrap(),
        matrix_homeserver_address: env::var("MATRIX_HOMESERVER_ADDRESS").unwrap(),
        matrix_channel_id: env::var("MATRIX_CHANNEL_ID").unwrap(),
        spaceapi_endpoint: env::var("SPACEAPI_ENDPOINT").unwrap(),
        matrix_topic_closed: env::var("MATRIX_TOPIC_CLOSED").unwrap(),
        matrix_topic_open: env::var("MATRIX_TOPIC_OPEN").unwrap(),
    }
}

async fn get_expected_topic(status: Status, config: &Config) -> String {
    if let Status::Open = status {
        config.matrix_topic_open.clone()
    } else {
        config.matrix_topic_closed.clone()
    }
}

async fn create_matrix_client(config: &Config) -> Client {
    let mut http_headers = header::HeaderMap::new();
    http_headers.insert(
        header::AUTHORIZATION,
        header::HeaderValue::from_str(&format!("Bearer {}", config.matrix_auth_token)).unwrap(),
    );

    let http_client = Client::builder()
        .user_agent(APP_USER_AGENT)
        .default_headers(http_headers)
        .build()
        .unwrap();

    return http_client;
}

#[tokio::main]
async fn main() {
    let config = read_config().await;

    let matrix_client = create_matrix_client(&config).await;

    // Get Status, Topic
    let api_status = get_api_status(&config).await;
    let channel_topic = get_channel_topic(&config, &matrix_client).await;

    // Get topic for status
    let expected_topic = get_expected_topic(api_status, &config).await;

    // Compare Status, Topic and change topic if needed
    if expected_topic == channel_topic {
        print!("Topic is correct")
    } else {
        set_channel_topic(expected_topic, &matrix_client, &config).await;
        print!("Topic was not correct and was changed")
    }
}
