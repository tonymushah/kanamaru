pub mod view;

use lanitra_manga_commons::*;

pub(crate) mod profiles {
    pub use lanitra_manga_profiles::*;
}

mod inner {
    kanamaru::include_proto!("mg.tonymushah.lanitra_manga.posts");
}

pub use inner::*;