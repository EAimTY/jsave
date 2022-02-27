use parking_lot::{
    MappedReentrantMutexGuard as InnerMappedReentrantMutexGuard,
    ReentrantMutex as InnerReentrantMutex, ReentrantMutexGuard as InnerReentrantMutexGuard,
};
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    fs::OpenOptions,
    io::Error,
    ops::Deref,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

pub struct ReentrantMutex<T: ?Sized> {
    file_path: PathBuf,
    data: InnerReentrantMutex<T>,
}

impl<T> ReentrantMutex<T>
where
    T: Serialize + for<'de> Deserialize<'de> + ?Sized,
{
    #[inline]
    pub fn init<P: Into<PathBuf>>(file_path: P) -> Result<Self, Error> {
        let file_path = file_path.into();

        let data = {
            let read = OpenOptions::new().read(true).open(&file_path)?;
            serde_json::from_reader(read)?
        };

        crate::save_data_to_path(&data, &file_path)?;

        Ok(Self {
            data: InnerReentrantMutex::new(data),
            file_path,
        })
    }

    #[inline]
    pub fn init_with<P: Into<PathBuf>>(data: T, file_path: P) -> Result<Self, Error> {
        let file_path = file_path.into();

        crate::save_data_to_path(&data, &file_path)?;

        Ok(Self {
            data: InnerReentrantMutex::new(data),
            file_path,
        })
    }

    #[inline]
    pub fn into_inner(self) -> T {
        self.data.into_inner()
    }

    #[inline]
    pub fn path(&self) -> &Path {
        &self.file_path
    }

    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        self.data.get_mut()
    }

    #[inline]
    pub fn is_locked(&self) -> bool {
        self.data.is_locked()
    }

    #[inline]
    pub fn is_owned_by_current_thread(&self) -> bool {
        self.data.is_owned_by_current_thread()
    }

    #[inline]
    pub fn data_ptr(&self) -> *mut T {
        self.data.data_ptr()
    }

    #[inline]
    pub fn lock(&self) -> ReentrantMutexGuard<T> {
        ReentrantMutexGuard {
            remutex: self,
            guard: self.data.lock(),
        }
    }

    #[inline]
    pub fn try_lock(&self) -> Option<ReentrantMutexGuard<T>> {
        self.data.try_lock().map(|g| ReentrantMutexGuard {
            remutex: self,
            guard: g,
        })
    }

    #[inline]
    pub fn try_lock_for(&self, timeout: Duration) -> Option<ReentrantMutexGuard<T>> {
        self.data
            .try_lock_for(timeout)
            .map(|g| ReentrantMutexGuard {
                remutex: self,
                guard: g,
            })
    }

    #[inline]
    pub fn try_lock_until(&self, timeout: Instant) -> Option<ReentrantMutexGuard<T>> {
        self.data
            .try_lock_until(timeout)
            .map(|g| ReentrantMutexGuard {
                remutex: self,
                guard: g,
            })
    }

    #[inline]
    pub fn save(&self) -> Result<(), Error> {
        let data = self.data.lock();
        crate::save_data_to_path(data.deref(), &self.file_path)
    }

    #[inline]
    pub fn try_save(&self) -> Option<Result<(), Error>> {
        self.data
            .try_lock()
            .map(|data| crate::save_data_to_path(data.deref(), &self.file_path))
    }

    #[inline]
    pub fn try_save_for(&self, timeout: Duration) -> Option<Result<(), Error>> {
        self.data
            .try_lock_for(timeout)
            .map(|data| crate::save_data_to_path(data.deref(), &self.file_path))
    }

    #[inline]
    pub fn try_save_until(&self, timeout: Instant) -> Option<Result<(), Error>> {
        self.data
            .try_lock_until(timeout)
            .map(|data| crate::save_data_to_path(data.deref(), &self.file_path))
    }

    #[inline]
    pub unsafe fn force_unlock(&self) {
        self.data.force_unlock()
    }

    #[inline]
    pub unsafe fn force_unlock_fair(&self) {
        self.data.force_unlock_fair()
    }
}

impl<T> Debug for ReentrantMutex<T>
where
    T: Debug + Serialize + for<'de> Deserialize<'de> + ?Sized,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.data.fmt(f)
    }
}

pub struct ReentrantMutexGuard<'a, T: ?Sized> {
    remutex: &'a ReentrantMutex<T>,
    guard: InnerReentrantMutexGuard<'a, T>,
}

impl<'a, T: ?Sized> ReentrantMutexGuard<'a, T> {
    #[inline]
    pub fn remutex(s: &Self) -> &'a ReentrantMutex<T> {
        s.remutex
    }

    #[inline]
    pub fn map<U: ?Sized, F>(s: Self, f: F) -> MappedReentrantMutexGuard<'a, U>
    where
        F: FnOnce(&T) -> &U,
    {
        MappedReentrantMutexGuard(InnerReentrantMutexGuard::map(s.guard, f))
    }

    #[inline]
    pub fn try_map<U: ?Sized, F>(s: Self, f: F) -> Result<MappedReentrantMutexGuard<'a, U>, Self>
    where
        F: FnOnce(&mut T) -> Option<&mut U>,
    {
        InnerReentrantMutexGuard::try_map(s.guard, f).map_or_else(
            |g| {
                Err(Self {
                    remutex: s.remutex,
                    guard: g,
                })
            },
            |g| Ok(MappedReentrantMutexGuard(g)),
        )
    }

    #[inline]
    pub fn unlocked<F, U>(s: &mut Self, f: F) -> U
    where
        F: FnOnce() -> U,
    {
        InnerReentrantMutexGuard::unlocked(&mut s.guard, f)
    }

    #[inline]
    pub fn unlocked_fair<F, U>(s: &mut Self, f: F) -> U
    where
        F: FnOnce() -> U,
    {
        InnerReentrantMutexGuard::unlocked_fair(&mut s.guard, f)
    }

    #[inline]
    pub fn unlock_fair(s: Self) {
        InnerReentrantMutexGuard::unlock_fair(s.guard);
    }

    #[inline]
    pub fn bump(s: &mut Self) {
        InnerReentrantMutexGuard::bump(&mut s.guard);
    }
}

impl<T> Debug for ReentrantMutexGuard<'_, T>
where
    T: Debug + ?Sized,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.guard.fmt(f)
    }
}

impl<T> Display for ReentrantMutexGuard<'_, T>
where
    T: Display + ?Sized,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.guard.fmt(f)
    }
}

impl<T: ?Sized + Serialize> Deref for ReentrantMutexGuard<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.guard.deref()
    }
}

pub struct MappedReentrantMutexGuard<'a, T: ?Sized>(InnerMappedReentrantMutexGuard<'a, T>);

impl<'a, T: ?Sized> MappedReentrantMutexGuard<'a, T> {
    #[inline]
    pub fn map<U: ?Sized, F>(s: Self, f: F) -> MappedReentrantMutexGuard<'a, U>
    where
        F: FnOnce(&T) -> &U,
    {
        MappedReentrantMutexGuard(InnerMappedReentrantMutexGuard::map(s.0, f))
    }

    #[inline]
    pub fn try_map<U: ?Sized, F>(s: Self, f: F) -> Result<MappedReentrantMutexGuard<'a, U>, Self>
    where
        F: FnOnce(&T) -> Option<&U>,
    {
        InnerMappedReentrantMutexGuard::try_map(s.0, f)
            .map_or_else(|g| Err(Self(g)), |g| Ok(MappedReentrantMutexGuard(g)))
    }

    #[inline]
    pub fn unlock_fair(s: Self) {
        InnerMappedReentrantMutexGuard::unlock_fair(s.0);
    }
}

impl<T> Debug for MappedReentrantMutexGuard<'_, T>
where
    T: Debug + ?Sized,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.0.fmt(f)
    }
}

impl<T> Display for MappedReentrantMutexGuard<'_, T>
where
    T: Display + ?Sized,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.0.fmt(f)
    }
}

impl<T: ?Sized + Serialize> Deref for MappedReentrantMutexGuard<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}
