#![doc = include_str!("../README.md")]

use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    ffi::{OsStr, OsString},
    fs::OpenOptions,
    io,
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
};
use thiserror::Error;

pub struct Jsave<T> {
    data: RwLock<T>,
    path: PathBuf,
}

impl<T> Jsave<T>
where
    T: Serialize + ?Sized + for<'de> Deserialize<'de>,
{
    pub fn init<P: IntoPathBuf>(path: P) -> Result<Self, Error> {
        let path = path.into_path_buf();

        let data = {
            let read = OpenOptions::new().read(true).open(&path)?;
            serde_json::from_reader(read)?
        };

        let file = OpenOptions::new().write(true).truncate(true).open(&path)?;
        serde_json::to_writer(file, &data)?;

        Ok(Jsave {
            data: RwLock::new(data),
            path,
        })
    }

    pub fn init_with<P: IntoPathBuf>(data: T, path: P) -> Result<Self, Error> {
        let path = path.into_path_buf();

        let file = OpenOptions::new().write(true).truncate(true).open(&path)?;
        serde_json::to_writer(file, &data)?;

        Ok(Jsave {
            data: RwLock::new(data),
            path,
        })
    }

    pub fn read(&self) -> JsaveReadGuard<T> {
        JsaveReadGuard {
            guard: self.data.read(),
        }
    }

    pub fn write(&self) -> JsaveWriteGuard<T> {
        JsaveWriteGuard {
            guard: self.data.write(),
            path: self.path.as_ref(),
        }
    }
}

pub struct JsaveReadGuard<'a, T: ?Sized> {
    guard: RwLockReadGuard<'a, T>,
}

impl<T: ?Sized> Deref for JsaveReadGuard<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.guard.deref()
    }
}

pub struct JsaveWriteGuard<'a, T: ?Sized + Serialize> {
    guard: RwLockWriteGuard<'a, T>,
    path: &'a Path,
}

impl<T: ?Sized + Serialize> Deref for JsaveWriteGuard<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.guard.deref()
    }
}

impl<T: ?Sized + Serialize> DerefMut for JsaveWriteGuard<'_, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        self.guard.deref_mut()
    }
}

impl<T: ?Sized + Serialize> Drop for JsaveWriteGuard<'_, T> {
    fn drop(&mut self) {
        if let Ok(file) = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(self.path)
        {
            let data = self.guard.deref();
            let _ = serde_json::to_writer(file, data);
        }
    }
}

pub trait IntoPathBuf {
    fn into_path_buf(self) -> PathBuf;
}

impl IntoPathBuf for PathBuf {
    #[inline]
    fn into_path_buf(self) -> PathBuf {
        self
    }
}

impl IntoPathBuf for &Path {
    #[inline]
    fn into_path_buf(self) -> PathBuf {
        self.to_owned()
    }
}

impl IntoPathBuf for Cow<'_, Path> {
    #[inline]
    fn into_path_buf(self) -> PathBuf {
        self.into_owned()
    }
}

impl IntoPathBuf for String {
    #[inline]
    fn into_path_buf(self) -> PathBuf {
        PathBuf::from(self)
    }
}

impl IntoPathBuf for &str {
    #[inline]
    fn into_path_buf(self) -> PathBuf {
        Path::new(self).to_owned()
    }
}

impl IntoPathBuf for Cow<'_, str> {
    #[inline]
    fn into_path_buf(self) -> PathBuf {
        PathBuf::from(self.into_owned())
    }
}

impl IntoPathBuf for OsString {
    #[inline]
    fn into_path_buf(self) -> PathBuf {
        PathBuf::from(self)
    }
}

impl IntoPathBuf for &OsStr {
    #[inline]
    fn into_path_buf(self) -> PathBuf {
        Path::new(self).to_owned()
    }
}

impl IntoPathBuf for Cow<'_, OsStr> {
    #[inline]
    fn into_path_buf(self) -> PathBuf {
        PathBuf::from(self.into_owned())
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
}
