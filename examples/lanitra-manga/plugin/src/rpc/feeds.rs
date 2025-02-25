use atrium_api::app::bsky::feed::{
    get_feed::{Parameters as GetFeedParameters, ParametersData as GetFeedParametersData},
    get_timeline::{
        Parameters as GetTimelineParameters, ParametersData as GetTimelineParametersData,
    },
};
use kanamaru::prelude::*;
use lanitra_manga_feeds::{
    feeds_responder::Feeds, GetFeedRequest, GetFeedResponse, GetHomeFeedRequest,
    GetTimelineRequest, GetTimelineResponse,
};
use serde::Deserialize;
use tauri::Runtime;

use crate::{client::GetBskyClient, PLUGIN_NAME};

#[derive(Debug)]
pub struct FeedsService;

#[async_trait]
impl Feeds for FeedsService {
    async fn get_timeline<R: Runtime>(
        &self,
        _request: UnaryRequest<R, GetTimelineRequest>,
    ) -> Result<UnaryResponse<GetTimelineResponse>, Status> {
        let webview = _request.webview();
        let agent = webview.get_bsky_client();
        let inner_req = _request.message().clone();
        let parameters: GetTimelineParameters = GetTimelineParametersData {
            algorithm: inner_req.algorithm,
            cursor: inner_req.cursor,
            limit: inner_req.limit.and_then(|i| (i as u8).try_into().ok()),
        }
        .into();
        let res = agent.api.app.bsky.feed.get_timeline(parameters).await?;
        Ok(UnaryResponse::new(GetTimelineResponse {
            feed: res
                .data
                .feed
                .into_iter()
                .map(|data| data.data.into())
                .collect(),
            cursor: res.data.cursor,
        }))
    }
    async fn get_feed<R: Runtime>(
        &self,
        _request: UnaryRequest<R, GetFeedRequest>,
    ) -> Result<UnaryResponse<GetFeedResponse>, Status> {
        let webview = _request.webview();
        let agent = webview.get_bsky_client();
        let inner_req = _request.message().clone();
        let parameters: GetFeedParameters = GetFeedParametersData {
            feed: inner_req.feed,
            cursor: inner_req.cursor,
            limit: inner_req.limit.and_then(|i| (i as u8).try_into().ok()),
        }
        .into();
        let res = agent.api.app.bsky.feed.get_feed(parameters).await?;
        Ok(UnaryResponse::new(GetFeedResponse {
            feed: res
                .data
                .feed
                .into_iter()
                .map(|data| data.data.into())
                .collect(),
            cursor: res.data.cursor,
        }))
    }
    async fn get_home_feed<R: Runtime>(
        &self,
        _request: UnaryRequest<R, GetHomeFeedRequest>,
    ) -> Result<UnaryResponse<GetFeedResponse>, Status> {
        let plugin_config = crate::Config::deserialize(
            _request
                .app_handle()
                .config()
                .plugins
                .0
                .get(PLUGIN_NAME)
                .cloned()
                .unwrap_or_default(),
        )?;
        let feed = plugin_config.home.feed;
        self.get_feed(_request.map_message(|message| GetFeedRequest {
            feed,
            cursor: message.cursor,
            limit: message.limit,
        }))
        .await
    }
}
