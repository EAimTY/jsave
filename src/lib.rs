#![doc = include_str!("../README.md")]

pub mod mutex;
pub mod rwlock;

pub use crate::{mutex::Mutex, rwlock::RwLock};

use serde::{Deserialize, Serialize};
use std::{fs::OpenOptions, io::Result, path::Path};

fn save_data_to_path<T>(data: &T, path: &Path) -> Result<()>
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
