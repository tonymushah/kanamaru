use kanamaru::prelude::*;
use lanitra_manga_feeds::{feeds_server::Feeds, GetTimelineRequest, GetTimelineResponse};
use tauri::Runtime;

#[derive(Debug)]
pub struct FeedsService;

#[async_trait]
impl Feeds for FeedsService {
    async fn get_timeline<R: Runtime>(
        &self,
        _request: UnaryRequest<R, GetTimelineRequest>,
    ) -> Result<UnaryResponse<GetTimelineResponse>, Status> {
        Err(Status::unimplemented("Not yet implemented"))
    }
}
