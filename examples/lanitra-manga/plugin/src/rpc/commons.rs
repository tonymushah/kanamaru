use std::pin::Pin;

use kanamaru::prelude::*;

use bsky_sdk::rich_text::RichText as SdkRichText;
use lanitra_manga_commons::{utils_responder::Utils, RichText, RichTextDetails};
use tauri::Runtime;
use tokio_stream::{Stream, StreamExt};

use crate::client::GetBskyClient;

pub struct UtilsService;

#[async_trait]
impl Utils for UtilsService {
    async fn get_rich_text_details<R: Runtime>(
        &self,
        request: UnaryRequest<R, RichText>,
    ) -> Result<UnaryResponse<RichTextDetails>, Status> {
        let app_handle = request.app_handle();
        let clients = app_handle.get_client_inner();
        let mut rich_text = SdkRichText::new(&request.message().text, None);
        if request.message().detect_facets.unwrap_or(true) {
            rich_text
                .detect_facets(clients.bsky_reqwest.clone())
                .await?;
        }
        Ok(UnaryResponse::new(RichTextDetails {
            markdown: {
                let mut markdown = String::new();
                for segment in rich_text.segments() {
                    if let Some(link) = segment.link() {
                        markdown += &format!("[{}]({})", segment.text, link.uri);
                    } else if let Some(mention) = segment.mention() {
                        markdown += &format!(
                            "[{}](https://bsky.app/profile/{})",
                            segment.text,
                            mention.did.as_str()
                        );
                    } else {
                        markdown += &segment.text;
                    }
                }
                markdown
            },
            length: rich_text.segments().len() as u64,
            grapheme_length: rich_text.grapheme_len() as u64,
        }))
    }
    type GetRichTextDetailsStreamStream = Pin<
        Box<dyn Stream<Item = Result<IpcMessage<RichTextDetails>, Status>> + Send + Sync + 'static>,
    >;

    async fn get_rich_text_details_stream<R: Runtime>(
        &self,
        mut request: StreamingRequest<R, RichText>,
    ) -> Result<StreamingResponse<RichTextDetails, Self::GetRichTextDetailsStreamStream>, Status>
    {
        let app_handle = request.app_handle();

        let stream = async_stream::try_stream! {
            let clients = app_handle.get_client_inner();
            while let Some(request) = request.stream_mut().next().await {
                let request = request?.body;
                let mut rich_text = SdkRichText::new(&request.text, None);
                if request.detect_facets.unwrap_or(true) {
                    rich_text
                        .detect_facets(clients.bsky_reqwest.clone())
                        .await?;
                }
                yield IpcMessage::new(RichTextDetails {
                    markdown: {
                        let mut markdown = String::new();
                        for segment in rich_text.segments() {
                            if let Some(link) = segment.link() {
                                markdown += &format!("[{}]({})", segment.text, link.uri);
                            } else if let Some(mention) = segment.mention() {
                                markdown += &format!(
                                    "[{}](https://bsky.app/profile/{})",
                                    segment.text,
                                    mention.did.as_str()
                                );
                            } else {
                                markdown += &segment.text;
                            }
                        }
                        markdown
                    },
                    length: rich_text.segments().len() as u64,
                    grapheme_length: rich_text.grapheme_len() as u64,
                });
            }
        };
        Ok(StreamingResponse::new(Box::pin(stream)))
    }
}
