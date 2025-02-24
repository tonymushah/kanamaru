use atrium_api::{
    app::bsky::feed::get_timeline::{
        Parameters as GetTimelineParameters, ParametersData as GetTimelineParametersData,
    },
    types::Object,
};
use kanamaru::prelude::*;
use lanitra_manga_feeds::{
    feeds_responder::Feeds, FeedViewPostMessage, GetTimelineRequest, GetTimelineResponse,
};
use tauri::{Manager, Runtime};

use crate::client::GetBskyClient;

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
            feed: res.data.feed.into_iter().map(|data| todo!()).collect(),
            cursor: res.data.cursor,
        }))
    }
}
