//! Package fixture bytes through the public codec for browser qualification.
use std::{io::Write, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 2 {
        return Err("usage: painted-asset <material.mix> <new-material.mixpack>".into());
    }
    let source = std::fs::read(PathBuf::from(&args[0]))?;
    let bytes = mixture_asset::write(&source, &[], &Default::default())?;
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(PathBuf::from(&args[1]))?
        .write_all(&bytes)?;
    Ok(())
}
