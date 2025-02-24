pub mod into_impl;

mod inner {
    kanamaru::include_proto!("mg.tonymushah.lanitra_manga.posts.view");
}

pub use inner::*;
