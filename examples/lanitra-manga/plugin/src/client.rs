use std::fmt::Debug;

use bsky_sdk::BskyAgent;
use derive_more::{Constructor, Deref, DerefMut, From};
use std::time::Duration;
use tauri::{AppHandle, Manager, Runtime, State};

use atrium_api::agent::atp_agent::store::MemorySessionStore;
use atrium_xrpc_client::reqwest::{ReqwestClient, ReqwestClientBuilder};
use bsky_sdk::agent::BskyAtpAgentBuilder;
use reqwest::{Client, ClientBuilder};
use tower::{buffer::BufferLayer, limit::RateLimitLayer, ServiceBuilder};

use crate::config::Config;

#[derive(Clone, Deref, DerefMut, From, Constructor)]
pub struct BskyClient(BskyAgent);

pub struct ReqwestInnerClient {
    pub bsky_reqwest: ReqwestClient,
    #[allow(dead_code)]
    pub client: Client,
}

impl ReqwestInnerClient {
    pub fn new(client: Client, config: &Config) -> Self {
        #[cfg(debug_assertions)]
        println!("{}", config.bsk_config.endpoint.as_str());
        Self {
            client: client.clone(),
            bsky_reqwest: ReqwestClientBuilder::new(config.bsk_config.endpoint.as_str())
                .client(client)
                .build(),
        }
    }
}

impl Debug for BskyClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BskyClient").finish()
    }
}

pub trait GetBskyClient<R>: Manager<R>
where
    R: Runtime,
{
    #[allow(dead_code)]
    fn try_get_bsky_client(&self) -> Option<State<'_, BskyClient>> {
        self.try_state()
    }
    fn try_get_client_inner(&self) -> Option<State<'_, ReqwestInnerClient>> {
        self.try_state()
    }
    #[allow(dead_code)]
    fn get_bsky_client(&self) -> State<'_, BskyClient> {
        self.try_get_bsky_client()
            .expect("The bskyClient is not initialized")
    }
    fn get_client_inner(&self) -> State<'_, ReqwestInnerClient> {
        self.try_get_client_inner()
            .expect("The client inners is not initialized")
    }
}

impl<M, R> GetBskyClient<R> for M
where
    M: Manager<R>,
    R: Runtime,
{
}

pub async fn setup_client<R: Runtime>(
    app: &AppHandle<R>,
    config: &Config,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = ClientBuilder::new()
        .user_agent(format!(
            "{}/{}",
            app.config().identifier,
            app.config().version.as_ref().map_or("0.0.0", |v| v)
        ))
        .connector_layer(
            ServiceBuilder::new()
                .layer(BufferLayer::new(1024))
                .layer(RateLimitLayer::new(
                    config.reqwest.rate_limit_number.unwrap_or(5),
                    config
                        .reqwest
                        .rate_limit_duration
                        .map(|d| -> Duration { d.into() })
                        .unwrap_or(Duration::from_secs(1)),
                )),
        )
        .build()?;
    let clients = ReqwestInnerClient::new(client, config);
    app.manage(clients);
    app.manage(BskyClient::new(
        BskyAtpAgentBuilder::new(app.state::<ReqwestInnerClient>().bsky_reqwest.clone())
            .store(MemorySessionStore::default())
            .config(config.bsk_config.clone())
            .build()
            .await?,
    ));
    Ok(())
}
