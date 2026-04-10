use include_dir::{Dir, include_dir};

pub static ASSETS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/assets");
