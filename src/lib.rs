#![doc = include_str!("../README.md")]

mod mutex;
pub mod rwlock;

pub use crate::rwlock::RwLock;

use serde::{Deserialize, Serialize};
use std::{fs::OpenOptions, io, path::Path};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
}

fn save_data_to_path<T>(data: &T, path: &Path) -> Result<(), Error>
where
    T: Serialize + for<'de> Deserialize<'de> + ?Sized,
{
    let file = OpenOptions::new().write(true).truncate(true).open(&path)?;

    #[cfg(feature = "pretty")]
    serde_json::to_writer_pretty(file, data)?;

    #[cfg(not(feature = "pretty"))]
    serde_json::to_writer(file, data)?;

    Ok(())
}
