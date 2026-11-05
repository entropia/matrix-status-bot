use anyhow::Context;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use std::env;
use std::time::Duration;
use tokio::try_join;
use tracing::{debug, info, instrument};

// TODO: Konfigurierbares Debug-Level

const APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);
const CLIENT_API: &str = "_matrix/client/v3";
#[derive(Deserialize, Debug, PartialEq)]
enum Status {
    Open,
    Closed,
}

#[derive(Debug)]
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

#[instrument(err)]
fn read_config() -> anyhow::Result<Config> {
    #![allow(clippy::needless_return)]

    return Ok(Config {
        matrix_auth_token: read_env_var("MATRIX_AUTH_TOKEN")?,
        matrix_homeserver_address: read_env_var("MATRIX_HOMESERVER_ADDRESS")?,
        matrix_channel_id: read_env_var("MATRIX_CHANNEL_ID")?,
        spaceapi_endpoint: read_env_var("SPACEAPI_ENDPOINT")?,
        matrix_topic_closed: read_env_var("MATRIX_TOPIC_CLOSED")?,
        matrix_topic_open: read_env_var("MATRIX_TOPIC_OPEN")?,
    });
}

fn read_env_var(name: &str) -> anyhow::Result<String> {
    #![allow(clippy::needless_return)]

    return env::var(name).with_context(|| format!("Failed to read env var {}", name));
}

#[instrument(err)]
fn create_http_client() -> anyhow::Result<Client> {
    #![allow(clippy::needless_return)]

    let http_client = Client::builder()
        .user_agent(APP_USER_AGENT)
        .timeout(Duration::from_secs(5))
        .build()
        .context("Failed to create shared http client")?;

    return Ok(http_client);
}

#[instrument(err)]
async fn get_api_status(config: &Config, http_client: &Client) -> anyhow::Result<Status> {
    #![allow(clippy::needless_return)]

    let res = http_client
        .get(&config.spaceapi_endpoint)
        .send()
        .await
        .context("Failed to contact SpaceAPI endpoint")?
        .error_for_status()
        .context("Failed to get status from SpaceAPI endpoint")?
        .json::<ApiResponse>()
        .await
        .context("Failed to parse response from SpaceAPI endpoint")?;

    return if res.state.open {
        Ok(Status::Open)
    } else {
        Ok(Status::Closed)
    };
}

#[instrument(err)]
async fn get_channel_topic(config: &Config, http_client: &Client) -> anyhow::Result<String> {
    #![allow(clippy::needless_return)]
    // TODO: Add handling for empty room topics

    let res = http_client
        .get(format!(
            "{}/{}/rooms/{}/state/m.room.topic",
            config.matrix_homeserver_address.trim_end_matches("/"),
            CLIENT_API,
            config.matrix_channel_id.as_str()
        ))
        .bearer_auth(&config.matrix_auth_token)
        .send()
        .await
        .context("Failed to contact Matrix API endpoint")?
        .error_for_status()
        .context("Failed to get status from Matrix API endpoint")?
        .json::<MatrixResponse>()
        .await
        .context("Failed to parse response from Matrix API endpoint")?;

    return Ok(res.topic);
}

fn get_expected_topic(status: Status, config: &Config) -> String {
    #![allow(clippy::needless_return)]

    return if let Status::Open = status {
        String::from(&config.matrix_topic_open)
    } else {
        String::from(&config.matrix_topic_closed)
    };
}

#[instrument(err)]
async fn set_channel_topic(
    topic: String,
    http_client: &Client,
    config: &Config,
) -> anyhow::Result<()> {
    http_client
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
        .bearer_auth(&config.matrix_auth_token)
        .send()
        .await
        .context("Failed to contact Matrix API endpoint")?
        .error_for_status()
        .context("Matrix API endpoint error")?;
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();

    info!("Reading Config");
    let config = read_config()?;

    debug!("Create shared HTTP Client");
    let http_client = create_http_client()?;

    info!("Get SpaceAPI Status");
    info!("Get current channel topic");
    let (api_status, channel_topic) = try_join!(
        get_api_status(&config, &http_client),
        get_channel_topic(&config, &http_client),
    )?;

    // Get topic for status
    let expected_topic = get_expected_topic(api_status, &config);

    // Compare Status, Topic and change topic if needed
    if expected_topic == channel_topic {
        info!("Topic is correct and was not changed")
    } else {
        info!("Set channel topic");
        set_channel_topic(expected_topic, &http_client, &config).await?;
    }
    Ok(())
}
