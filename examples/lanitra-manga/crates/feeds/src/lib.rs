// use lanitra_manga_commons::*;

pub(crate) mod profiles {
    pub use lanitra_manga_profiles::*;
}

pub(crate) mod posts {
    
    pub use lanitra_manga_posts::*;
}

mod inner {
    kanamaru::include_proto!("mg.tonymushah.lanitra_manga.feeds");
}

pub use inner::*;