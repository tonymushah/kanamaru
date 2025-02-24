// use lanitra_manga_commons::*;

pub mod into_impl;

mod inner {
    kanamaru::include_proto!("mg.tonymushah.lanitra_manga.profiles");
}

pub use inner::*;
