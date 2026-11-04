use reqwest::{Client, header};
use serde::Deserialize;
use serde_json::json;
use std::env;
use std::time::Duration;
use tracing::{debug, info};

// TODO: Konfigurierbares Debug-Level

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
    #![allow(clippy::needless_return)]

    info!("Getting SpaceAPI status");

    let response = reqwest::get(config.spaceapi_endpoint.clone())
        .await
        .unwrap()
        .json::<ApiResponse>()
        .await
        .unwrap();
    return if response.state.open {
        Status::Open
    } else {
        Status::Closed
    };
}

async fn get_channel_topic(config: &Config, http_client: &Client) -> String {
    #![allow(clippy::needless_return)]

    info!("Getting current Channel Topic");

    let res = http_client.get(format!(
        "{}/{}/rooms/{}/state/m.room.topic",
        config
            .matrix_homeserver_address
            .clone()
            .trim_end_matches("/"),
        CLIENT_API,
        config.matrix_channel_id.as_str()
    ));
    return res
        .send()
        .await
        .unwrap()
        .json::<MatrixResponse>()
        .await
        .unwrap()
        .topic;
}

async fn set_channel_topic(topic: String, http_client: &Client, config: &Config) {
    debug!("Setting Channel Topic");

    let res = http_client
        .put(format!(
            "{}/{}/rooms/{}/state/m.room.topic",
            config
                .matrix_homeserver_address
                .clone()
                .trim_end_matches("/"),
            CLIENT_API,
            config.matrix_channel_id.as_str()
        ))
        .json(&json!({"topic": &topic}))
        .send()
        .await
        .unwrap();
    res.error_for_status().unwrap();
}

fn read_config() -> Config {
    #![allow(clippy::needless_return)]

    info!("Reading config");

    return Config {
        matrix_auth_token: env::var("MATRIX_AUTH_TOKEN").unwrap(),
        matrix_homeserver_address: env::var("MATRIX_HOMESERVER_ADDRESS").unwrap(),
        matrix_channel_id: env::var("MATRIX_CHANNEL_ID").unwrap(),
        spaceapi_endpoint: env::var("SPACEAPI_ENDPOINT").unwrap(),
        matrix_topic_closed: env::var("MATRIX_TOPIC_CLOSED").unwrap(),
        matrix_topic_open: env::var("MATRIX_TOPIC_OPEN").unwrap(),
    };
}

fn get_expected_topic(status: Status, config: &Config) -> String {
    #![allow(clippy::needless_return)]

    return if let Status::Open = status {
        config.matrix_topic_open.clone()
    } else {
        config.matrix_topic_closed.clone()
    };
}

fn create_matrix_client(config: &Config) -> Client {
    #![allow(clippy::needless_return)]

    debug!("Creating shared matrix client");

    let mut http_headers = header::HeaderMap::new();
    http_headers.insert(
        header::AUTHORIZATION,
        header::HeaderValue::from_str(&format!("Bearer {}", config.matrix_auth_token)).unwrap(),
    );

    let http_client = Client::builder()
        .user_agent(APP_USER_AGENT)
        .default_headers(http_headers)
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();

    return http_client;
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let config = read_config();

    let matrix_client = create_matrix_client(&config);

    // Get Status, Topic
    let api_status = get_api_status(&config).await;
    let channel_topic = get_channel_topic(&config, &matrix_client).await;

    // Get topic for status
    let expected_topic = get_expected_topic(api_status, &config);

    // Compare Status, Topic and change topic if needed
    if expected_topic == channel_topic {
        info!("Topic is correct and was not changed")
    } else {
        set_channel_topic(expected_topic, &matrix_client, &config).await;
        info!("Topic was not correct and was changed")
    }
}
