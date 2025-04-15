pub struct BuildConfig {
    pub commons: bool,
    pub profiles: bool,
    pub auth: bool,
    pub posts: bool,
    pub feeds: bool,
    pub posts_view: bool,
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            commons: true,
            profiles: true,
            auth: true,
            posts: true,
            feeds: true,
            posts_view: true,
        }
    }
}

impl BuildConfig {
    pub fn builder(self) -> kanamaru_build::ProstBuilder {
        let mut builder = prost_builder();
        
        if self.profiles {
            builder = builder.extern_path(
                ".mg.tonymushah.lanitra_manga.profiles",
                "::lanitra_manga_profiles",
            );
        }
        if self.auth {
            builder =
                builder.extern_path(".mg.tonymushah.lanitra_manga.auth", "::lanitra_manga_auth");
        }
        if self.posts {
            builder = builder.extern_path(
                ".mg.tonymushah.lanitra_manga.posts",
                "::lanitra_manga_posts",
            );
        }
        if self.feeds {
            builder = builder.extern_path(
                ".mg.tonymushah.lanitra_manga.feeds",
                "::lanitra_manga_feeds",
            );
        }
        if self.posts_view {
            builder = builder.extern_path(
                ".mg.tonymushah.lanitra_manga.posts.view",
                "::lanitra_manga_posts::view",
            );
        }
        if self.commons {
            builder =
                builder.extern_path(".mg.tonymushah.lanitra_manga.commons", "::lanitra_manga_commons");
        }
        builder
    }
}

pub fn builder() -> kanamaru_build::ProstBuilder {
    BuildConfig::default().builder()
}

pub fn prost_builder() -> kanamaru_build::ProstBuilder {
    kanamaru_build::ProstBuilder::default()
}
