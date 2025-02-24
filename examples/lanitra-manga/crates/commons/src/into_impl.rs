use std::str::FromStr;

use atrium_api::{
    com::atproto::repo::{
        create_record::OutputData as CreateRecordOutput, defs::CommitMetaData,
        put_record::OutputData as PutRecordOutput, strong_ref::MainData,
    },
    types::string::Cid,
};
use bsky_sdk::rich_text::RichText;

impl From<&crate::RichText> for RichText {
    fn from(value: &crate::RichText) -> Self {
        Self::new(&value.text, None)
    }
}

pub fn get_rich_text_details(rich_text: &RichText, base_url: &str) -> crate::RichTextDetails {
    crate::RichTextDetails {
        markdown: {
            let mut markdown = String::new();
            for segment in rich_text.segments() {
                if let Some(link) = segment.link() {
                    markdown += &format!("[{}]({})", segment.text, link.uri);
                } else if let Some(mention) = segment.mention() {
                    markdown += &format!("[{}]({base_url}{})", segment.text, mention.did.as_str());
                } else {
                    markdown += &segment.text;
                }
            }
            markdown
        },
        length: rich_text.segments().len() as u64,
        grapheme_length: rich_text.grapheme_len() as u64,
    }
}

impl From<MainData> for crate::MainData {
    fn from(value: MainData) -> Self {
        Self {
            cid: value.cid.as_ref().to_string(),
            uri: value.uri,
        }
    }
}

impl TryFrom<crate::MainData> for MainData {
    type Error = <Cid as FromStr>::Err;
    fn try_from(value: crate::MainData) -> Result<Self, Self::Error> {
        Ok(Self {
            cid: value.cid.parse()?,
            uri: value.uri,
        })
    }
}

impl From<MainData> for crate::DataRef {
    fn from(value: MainData) -> Self {
        Self {
            cid: value.cid.as_ref().to_string(),
            uri: value.uri,
            commit: None,
            validation_status: None,
        }
    }
}

impl TryFrom<crate::DataRef> for MainData {
    type Error = <Cid as FromStr>::Err;
    fn try_from(value: crate::DataRef) -> Result<Self, Self::Error> {
        Ok(Self {
            cid: value.cid.parse()?,
            uri: value.uri,
        })
    }
}

impl TryFrom<crate::CommitMetadata> for CommitMetaData {
    type Error = <Cid as FromStr>::Err;
    fn try_from(value: crate::CommitMetadata) -> Result<Self, Self::Error> {
        Ok(Self {
            cid: value.cid.parse()?,
            rev: value.rev,
        })
    }
}

impl From<CommitMetaData> for crate::CommitMetadata {
    fn from(value: CommitMetaData) -> Self {
        Self {
            cid: value.cid.as_ref().to_string(),
            rev: value.rev,
        }
    }
}

impl From<CreateRecordOutput> for crate::DataRef {
    fn from(value: CreateRecordOutput) -> Self {
        Self {
            cid: value.cid.as_ref().to_string(),
            uri: value.uri,
            commit: value.commit.map(|i| i.data.into()),
            validation_status: value.validation_status,
        }
    }
}

impl From<PutRecordOutput> for crate::DataRef {
    fn from(value: PutRecordOutput) -> Self {
        Self {
            cid: value.cid.as_ref().to_string(),
            uri: value.uri,
            commit: value.commit.map(|i| i.data.into()),
            validation_status: value.validation_status,
        }
    }
}
