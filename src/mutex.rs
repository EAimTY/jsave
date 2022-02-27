use parking_lot::{
    MappedMutexGuard as InnerMappedMutexGuard, Mutex as InnerMutex, MutexGuard as InnerMutexGuard,
};
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    fs::OpenOptions,
    io::Error,
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

pub struct Mutex<T: ?Sized> {
    file_path: PathBuf,
    data: InnerMutex<T>,
}

impl<T> Mutex<T>
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
            data: InnerMutex::new(data),
            file_path,
        })
    }

    #[inline]
    pub fn init_with<P: Into<PathBuf>>(data: T, file_path: P) -> Result<Self, Error> {
        let file_path = file_path.into();

        crate::save_data_to_path(&data, &file_path)?;

        Ok(Self {
            data: InnerMutex::new(data),
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
    pub fn data_ptr(&self) -> *mut T {
        self.data.data_ptr()
    }

    #[inline]
    pub fn lock(&self) -> MutexGuard<T> {
        MutexGuard {
            mutex: self,
            guard: self.data.lock(),
        }
    }

    #[inline]
    pub fn try_lock(&self) -> Option<MutexGuard<T>> {
        self.data.try_lock().map(|g| MutexGuard {
            mutex: self,
            guard: g,
        })
    }

    #[inline]
    pub fn try_lock_for(&self, timeout: Duration) -> Option<MutexGuard<T>> {
        self.data.try_lock_for(timeout).map(|g| MutexGuard {
            mutex: self,
            guard: g,
        })
    }

    #[inline]
    pub fn try_lock_until(&self, timeout: Instant) -> Option<MutexGuard<T>> {
        self.data.try_lock_until(timeout).map(|g| MutexGuard {
            mutex: self,
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
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn force_unlock(&self) {
        self.data.force_unlock()
    }

    #[inline]
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn force_unlock_fair(&self) {
        self.data.force_unlock_fair()
    }
}

impl<T> Debug for Mutex<T>
where
    T: Debug + Serialize + for<'de> Deserialize<'de> + ?Sized,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.data.fmt(f)
    }
}

pub struct MutexGuard<'a, T: ?Sized> {
    mutex: &'a Mutex<T>,
    guard: InnerMutexGuard<'a, T>,
}

impl<'a, T: ?Sized> MutexGuard<'a, T> {
    #[inline]
    pub fn mutex(s: &Self) -> &'a Mutex<T> {
        s.mutex
    }

    #[inline]
    pub fn map<U: ?Sized, F>(s: Self, f: F) -> MappedMutexGuard<'a, U>
    where
        F: FnOnce(&mut T) -> &mut U,
    {
        MappedMutexGuard(InnerMutexGuard::map(s.guard, f))
    }

    #[inline]
    pub fn try_map<U: ?Sized, F>(s: Self, f: F) -> Result<MappedMutexGuard<'a, U>, Self>
    where
        F: FnOnce(&mut T) -> Option<&mut U>,
    {
        InnerMutexGuard::try_map(s.guard, f).map_or_else(
            |g| {
                Err(Self {
                    mutex: s.mutex,
                    guard: g,
                })
            },
            |g| Ok(MappedMutexGuard(g)),
        )
    }

    #[inline]
    pub fn unlocked<F, U>(s: &mut Self, f: F) -> U
    where
        F: FnOnce() -> U,
    {
        InnerMutexGuard::unlocked(&mut s.guard, f)
    }

    #[inline]
    pub fn unlocked_fair<F, U>(s: &mut Self, f: F) -> U
    where
        F: FnOnce() -> U,
    {
        InnerMutexGuard::unlocked_fair(&mut s.guard, f)
    }

    #[inline]
    pub fn unlock_fair(s: Self) {
        InnerMutexGuard::unlock_fair(s.guard);
    }

    #[inline]
    pub fn bump(s: &mut Self) {
        InnerMutexGuard::bump(&mut s.guard);
    }
}

impl<T> Debug for MutexGuard<'_, T>
where
    T: Debug + ?Sized,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.guard.fmt(f)
    }
}

impl<T> Display for MutexGuard<'_, T>
where
    T: Display + ?Sized,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.guard.fmt(f)
    }
}

impl<T: ?Sized + Serialize> Deref for MutexGuard<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.guard.deref()
    }
}

impl<T: ?Sized + Serialize> DerefMut for MutexGuard<'_, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        self.guard.deref_mut()
    }
}

pub struct MappedMutexGuard<'a, T: ?Sized>(InnerMappedMutexGuard<'a, T>);

impl<'a, T: ?Sized> MappedMutexGuard<'a, T> {
    #[inline]
    pub fn map<U: ?Sized, F>(s: Self, f: F) -> MappedMutexGuard<'a, U>
    where
        F: FnOnce(&mut T) -> &mut U,
    {
        MappedMutexGuard(InnerMappedMutexGuard::map(s.0, f))
    }

    #[inline]
    pub fn try_map<U: ?Sized, F>(s: Self, f: F) -> Result<MappedMutexGuard<'a, U>, Self>
    where
        F: FnOnce(&mut T) -> Option<&mut U>,
    {
        InnerMappedMutexGuard::try_map(s.0, f)
            .map_or_else(|g| Err(Self(g)), |g| Ok(MappedMutexGuard(g)))
    }

    #[inline]
    pub fn unlock_fair(s: Self) {
        InnerMappedMutexGuard::unlock_fair(s.0);
    }
}

impl<T> Debug for MappedMutexGuard<'_, T>
where
    T: Debug + ?Sized,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.0.fmt(f)
    }
}

impl<T> Display for MappedMutexGuard<'_, T>
where
    T: Display + ?Sized,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.0.fmt(f)
    }
}

impl<T: ?Sized + Serialize> Deref for MappedMutexGuard<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl<T: ?Sized + Serialize> DerefMut for MappedMutexGuard<'_, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        self.0.deref_mut()
    }
}
