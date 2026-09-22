//! Author consumer-owned fixtures using only the public codec, outside the producer.
#[path = "../tests/asset_support/mod.rs"]
mod support;
fn main() {
    let path = std::path::PathBuf::from(std::env::args_os().nth(1).expect("fresh destination"));
    std::fs::create_dir(&path).unwrap();
    for size in support::SIZES {
        let bytes = support::archive(size);
        std::fs::write(path.join(format!("{}x{}.mixpack", size[0], size[1])), bytes).unwrap();
    }
}
