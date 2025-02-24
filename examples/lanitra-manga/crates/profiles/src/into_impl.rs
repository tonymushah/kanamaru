use atrium_api::app::bsky::actor::defs::{
    ProfileViewBasicData as DefsProfileViewBasicData, ProfileViewData, ProfileViewDetailedData,
};

impl From<DefsProfileViewBasicData> for crate::ProfileViewBasic {
    fn from(value: DefsProfileViewBasicData) -> Self {
        Self {
            did: value.did.to_string(),
            handle: value.handle.into(),
            display_name: value.display_name,
            created_at: value.created_at.map(|t| t.as_str().into()),
            avatar: value.avatar,
            description: None,
        }
    }
}

impl From<ProfileViewDetailedData> for crate::ProfileViewBasic {
    fn from(value: ProfileViewDetailedData) -> Self {
        Self {
            did: value.did.to_string(),
            handle: value.handle.into(),
            display_name: value.display_name,
            created_at: value.created_at.map(|t| t.as_str().into()),
            avatar: value.avatar,
            description: value.description,
        }
    }
}

impl From<ProfileViewData> for crate::ProfileViewBasic {
    fn from(value: ProfileViewData) -> Self {
        Self {
            did: value.did.to_string(),
            handle: value.handle.into(),
            display_name: value.display_name,
            created_at: value.created_at.map(|t| t.as_str().into()),
            avatar: value.avatar,
            description: value.description,
        }
    }
}
