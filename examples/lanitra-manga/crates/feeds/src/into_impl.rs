use atrium_api::{
    app::bsky::feed::defs::{FeedViewPostData, FeedViewPostReasonRefs, ReasonRepostData},
    types::Union,
};

use crate::feed_view_post_reason::Reason;

impl From<ReasonRepostData> for super::ReasonRepost {
    fn from(value: ReasonRepostData) -> Self {
        Self {
            by: Some(value.by.data.into()),
            indexed_at: value.indexed_at.as_str().into(),
        }
    }
}

impl From<FeedViewPostReasonRefs> for super::FeedViewPostReason {
    fn from(value: FeedViewPostReasonRefs) -> Self {
        match value {
            FeedViewPostReasonRefs::ReasonRepost(object) => Self {
                reason: Some(Reason::Repost(object.data.into())),
            },
            FeedViewPostReasonRefs::ReasonPin(_) => Self {
                reason: Some(Reason::Pin(crate::ReasonPin {})),
            },
        }
    }
}

impl From<FeedViewPostData> for super::FeedViewPostMessage {
    fn from(value: FeedViewPostData) -> Self {
        Self {
            feed_context: value.feed_context,
            post: Some(value.post.data.into()),
            reply: value.reply.map(|reply| reply.data.into()),
            reason: match value.reason {
                Some(Union::Refs(reason)) => Some(reason.into()),
                _ => None,
            },
        }
    }
}
