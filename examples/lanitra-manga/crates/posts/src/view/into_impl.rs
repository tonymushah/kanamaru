use atrium_api::{
    app::bsky::{
        embed::{
            external::ViewExternalData,
            images::{ViewData as ViewImagesData, ViewImageData},
            video::ViewData as ViewVideoData,
        },
        feed::{
            defs::{
                BlockedPostData, NotFoundPostData, PostViewData, PostViewEmbedRefs, ReplyRefData,
                ReplyRefParentRefs, ReplyRefRootRefs, ThreadViewPostData, ThreadViewPostParentRefs,
                ThreadViewPostRepliesItem,
            },
            get_post_thread::OutputThreadRefs,
            post::RecordData as PostRecordData,
        },
    },
    types::{TryFromUnknown, Union},
};
use bsky_sdk::rich_text::RichText;
use lanitra_manga_commons::Empty;

use super::{thread_view_post_inner::ActualPost, ThreadViewPostInner};

impl From<ViewExternalData> for super::ViewExternal {
    fn from(value: ViewExternalData) -> Self {
        Self {
            description: value.description,
            title: value.title,
            uri: value.uri,
            thumb: value.thumb,
        }
    }
}

impl From<ViewVideoData> for super::ViewVideo {
    fn from(value: ViewVideoData) -> Self {
        Self {
            alt: value.alt,
            cid: value.cid.as_ref().to_string(),
            playlist: value.playlist,
            thumbnail: value.thumbnail,
            ratio: value.aspect_ratio.map(Into::into),
        }
    }
}

impl From<ViewImageData> for super::ViewImage {
    fn from(value: ViewImageData) -> Self {
        Self {
            fullsize: value.fullsize,
            thumb: value.thumb,
            alt: value.alt,
            ratio: value.aspect_ratio.map(Into::into),
        }
    }
}

impl From<ViewImagesData> for super::ViewImages {
    fn from(value: ViewImagesData) -> Self {
        Self {
            images: value
                .images
                .into_iter()
                .map(|o| o.data)
                .map(Into::into)
                .collect(),
        }
    }
}

impl From<PostViewEmbedRefs> for super::post_view_message::Embed {
    fn from(value: PostViewEmbedRefs) -> Self {
        match value {
            PostViewEmbedRefs::AppBskyEmbedImagesView(object) => Self::Images(object.data.into()),
            PostViewEmbedRefs::AppBskyEmbedVideoView(object) => Self::Video(object.data.into()),
            PostViewEmbedRefs::AppBskyEmbedExternalView(object) => {
                Self::External(object.data.external.data.into())
            }
            _ => Self::Others(Empty {}),
        }
    }
}

impl From<PostViewData> for super::PostViewMessage {
    fn from(value: PostViewData) -> Self {
        Self {
            cid: value.cid.as_ref().to_string(),
            uri: value.uri,
            author: Some(value.author.data.into()),
            like_count: value.like_count,
            quote_count: value.quote_count,
            reply_count: value.reply_count,
            repost_cound: value.repost_count,
            indexed_at: value.indexed_at.as_str().into(),
            content: PostRecordData::try_from_unknown(value.record.clone())
                .ok()
                .map(|content| RichText::new(&content.text, content.facets.clone()).into()),
            embed: match value.embed {
                Some(Union::Refs(embed)) => Some(embed.into()),
                _ => None,
            },
        }
    }
}

impl From<NotFoundPostData> for super::NotFoundPost {
    fn from(value: NotFoundPostData) -> Self {
        Self {
            uri: value.uri,
            not_found: value.not_found,
        }
    }
}

impl From<BlockedPostData> for super::BlockedPost {
    fn from(value: BlockedPostData) -> Self {
        Self {
            uri: value.uri,
            blocked: value.blocked,
            author_did: value.author.data.did.into(),
        }
    }
}

impl From<ThreadViewPostParentRefs> for super::ThreadViewPostInner {
    fn from(value: ThreadViewPostParentRefs) -> Self {
        let actual_post: ActualPost = match value {
            ThreadViewPostParentRefs::ThreadViewPost(object) => {
                ActualPost::Post(Box::new(object.data.into()))
            }
            ThreadViewPostParentRefs::NotFoundPost(object) => {
                ActualPost::NotFound(object.data.into())
            }
            ThreadViewPostParentRefs::BlockedPost(object) => {
                ActualPost::BlockedPost(object.data.into())
            }
        };
        Self {
            actual_post: Some(actual_post),
        }
    }
}

impl From<ThreadViewPostRepliesItem> for super::ThreadViewPostInner {
    fn from(value: ThreadViewPostRepliesItem) -> Self {
        let actual_post: ActualPost = match value {
            ThreadViewPostRepliesItem::ThreadViewPost(object) => {
                ActualPost::Post(Box::new(object.data.into()))
            }
            ThreadViewPostRepliesItem::NotFoundPost(object) => {
                ActualPost::NotFound(object.data.into())
            }
            ThreadViewPostRepliesItem::BlockedPost(object) => {
                ActualPost::BlockedPost(object.data.into())
            }
        };
        Self {
            actual_post: Some(actual_post),
        }
    }
}

impl From<ThreadViewPostData> for super::TheardViewPost {
    fn from(value: ThreadViewPostData) -> Self {
        Self {
            parent: match value.parent {
                Some(Union::Refs(parent)) => Some(Box::new(parent.into())),
                _ => None,
            },
            post: Some(value.post.data.into()),
            replies: match value.replies {
                Some(replies) => replies
                    .into_iter()
                    .flat_map(|reply| match reply {
                        Union::Refs(data) => Some(data.into()),
                        Union::Unknown(_) => None,
                    })
                    .collect(),
                _ => Default::default(),
            },
        }
    }
}

impl From<OutputThreadRefs> for super::ViewThreadResponse {
    fn from(value: OutputThreadRefs) -> Self {
        match value {
            OutputThreadRefs::AppBskyFeedDefsThreadViewPost(object) => Self {
                thread: Some(ThreadViewPostInner {
                    actual_post: Some(ActualPost::Post(Box::new(object.data.into()))),
                }),
            },
            OutputThreadRefs::AppBskyFeedDefsNotFoundPost(object) => Self {
                thread: Some(ThreadViewPostInner {
                    actual_post: Some(ActualPost::NotFound(object.data.into())),
                }),
            },
            OutputThreadRefs::AppBskyFeedDefsBlockedPost(object) => Self {
                thread: Some(ThreadViewPostInner {
                    actual_post: Some(ActualPost::BlockedPost(object.data.into())),
                }),
            },
        }
    }
}

impl From<ReplyRefRootRefs> for super::ThreadViewPostInner {
    fn from(value: ReplyRefRootRefs) -> Self {
        match value {
            ReplyRefRootRefs::PostView(object) => Self {
                actual_post: Some(ActualPost::Post(Box::new(super::TheardViewPost {
                    parent: None,
                    post: Some(object.data.into()),
                    replies: Default::default(),
                }))),
            },
            ReplyRefRootRefs::NotFoundPost(object) => Self {
                actual_post: Some(ActualPost::NotFound(object.data.into())),
            },
            ReplyRefRootRefs::BlockedPost(object) => Self {
                actual_post: Some(ActualPost::BlockedPost(object.data.into())),
            },
        }
    }
}

impl From<ReplyRefParentRefs> for super::ThreadViewPostInner {
    fn from(value: ReplyRefParentRefs) -> Self {
        match value {
            ReplyRefParentRefs::PostView(object) => Self {
                actual_post: Some(ActualPost::Post(Box::new(super::TheardViewPost {
                    parent: None,
                    post: Some(object.data.into()),
                    replies: Default::default(),
                }))),
            },
            ReplyRefParentRefs::NotFoundPost(object) => Self {
                actual_post: Some(ActualPost::NotFound(object.data.into())),
            },
            ReplyRefParentRefs::BlockedPost(object) => Self {
                actual_post: Some(ActualPost::BlockedPost(object.data.into())),
            },
        }
    }
}

impl From<ReplyRefData> for super::ReplyRefMessage {
    fn from(value: ReplyRefData) -> Self {
        Self {
            root: match value.root {
                Union::Refs(data) => Some(data.into()),
                Union::Unknown(_) => None,
            },
            parent: match value.parent {
                Union::Refs(data) => Some(data.into()),
                Union::Unknown(_) => None,
            },
            grandparent_author: value.grandparent_author.map(|parent| parent.data.into()),
        }
    }
}
