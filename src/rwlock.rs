use crate::Error;
use parking_lot::{
    MappedRwLockReadGuard as InnerMappedRwLockReadGuard,
    MappedRwLockWriteGuard as InnerMappedRwLockWriteGuard, RwLock as InnerRwLock,
    RwLockReadGuard as InnerRwLockReadGuard,
    RwLockUpgradableReadGuard as InnerRwLockUpgradableReadGuard,
    RwLockWriteGuard as InnerRwLockWriteGuard,
};
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    fs::OpenOptions,
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

pub struct RwLock<T: ?Sized> {
    file_path: PathBuf,
    data: InnerRwLock<T>,
}

impl<T> RwLock<T>
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
            data: InnerRwLock::new(data),
            file_path,
        })
    }

    #[inline]
    pub fn init_with<P: Into<PathBuf>>(data: T, file_path: P) -> Result<Self, Error> {
        let file_path = file_path.into();

        crate::save_data_to_path(&data, &file_path)?;

        Ok(Self {
            data: InnerRwLock::new(data),
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
    pub fn is_locked_exclusive(&self) -> bool {
        self.data.is_locked_exclusive()
    }

    #[inline]
    pub fn data_ptr(&self) -> *mut T {
        self.data.data_ptr()
    }

    #[inline]
    pub fn read(&self) -> RwLockReadGuard<T> {
        RwLockReadGuard {
            rwlock: self,
            guard: self.data.read(),
        }
    }

    #[inline]
    pub fn try_read(&self) -> Option<RwLockReadGuard<T>> {
        self.data.try_read().map(|g| RwLockReadGuard {
            rwlock: self,
            guard: g,
        })
    }

    #[inline]
    pub fn try_read_for(&self, timeout: Duration) -> Option<RwLockReadGuard<T>> {
        self.data.try_read_for(timeout).map(|g| RwLockReadGuard {
            rwlock: self,
            guard: g,
        })
    }

    #[inline]
    pub fn try_read_until(&self, timeout: Instant) -> Option<RwLockReadGuard<T>> {
        self.data.try_read_until(timeout).map(|g| RwLockReadGuard {
            rwlock: self,
            guard: g,
        })
    }

    #[inline]
    pub fn read_recursive(&self) -> RwLockReadGuard<T> {
        RwLockReadGuard {
            rwlock: self,
            guard: self.data.read_recursive(),
        }
    }

    #[inline]
    pub fn try_read_recursive(&self) -> Option<RwLockReadGuard<T>> {
        self.data.try_read_recursive().map(|g| RwLockReadGuard {
            rwlock: self,
            guard: g,
        })
    }

    #[inline]
    pub fn try_read_recursive_for(&self, timeout: Duration) -> Option<RwLockReadGuard<T>> {
        self.data
            .try_read_recursive_for(timeout)
            .map(|g| RwLockReadGuard {
                rwlock: self,
                guard: g,
            })
    }

    #[inline]
    pub fn try_read_recursive_until(&self, timeout: Instant) -> Option<RwLockReadGuard<T>> {
        self.data
            .try_read_recursive_until(timeout)
            .map(|g| RwLockReadGuard {
                rwlock: self,
                guard: g,
            })
    }

    #[inline]
    pub fn write(&self) -> RwLockWriteGuard<T> {
        RwLockWriteGuard {
            rwlock: self,
            guard: self.data.write(),
        }
    }

    #[inline]
    pub fn try_write(&self) -> Option<RwLockWriteGuard<T>> {
        self.data.try_write().map(|g| RwLockWriteGuard {
            rwlock: self,
            guard: g,
        })
    }

    #[inline]
    pub fn try_write_for(&self, timeout: Duration) -> Option<RwLockWriteGuard<T>> {
        self.data.try_write_for(timeout).map(|g| RwLockWriteGuard {
            rwlock: self,
            guard: g,
        })
    }

    #[inline]
    pub fn try_write_until(&self, timeout: Instant) -> Option<RwLockWriteGuard<T>> {
        self.data
            .try_write_until(timeout)
            .map(|g| RwLockWriteGuard {
                rwlock: self,
                guard: g,
            })
    }

    #[inline]
    pub fn save(&self) -> Result<(), Error> {
        let data = self.data.write();
        crate::save_data_to_path(data.deref(), &self.file_path)
    }

    #[inline]
    pub fn try_save(&self) -> Option<Result<(), Error>> {
        self.data
            .try_write()
            .map(|data| crate::save_data_to_path(data.deref(), &self.file_path))
    }

    #[inline]
    pub fn try_save_for(&self, timeout: Duration) -> Option<Result<(), Error>> {
        self.data
            .try_write_for(timeout)
            .map(|data| crate::save_data_to_path(data.deref(), &self.file_path))
    }

    #[inline]
    pub fn try_save_until(&self, timeout: Instant) -> Option<Result<(), Error>> {
        self.data
            .try_write_until(timeout)
            .map(|data| crate::save_data_to_path(data.deref(), &self.file_path))
    }

    #[inline]
    pub fn upgradable_read(&self) -> RwLockUpgradableReadGuard<T> {
        RwLockUpgradableReadGuard {
            rwlock: self,
            guard: self.data.upgradable_read(),
        }
    }

    #[inline]
    pub fn try_upgradable_read(&self) -> Option<RwLockUpgradableReadGuard<T>> {
        self.data
            .try_upgradable_read()
            .map(|g| RwLockUpgradableReadGuard {
                rwlock: self,
                guard: g,
            })
    }

    #[inline]
    pub fn try_upgradable_read_for(
        &self,
        timeout: Duration,
    ) -> Option<RwLockUpgradableReadGuard<T>> {
        self.data
            .try_upgradable_read_for(timeout)
            .map(|g| RwLockUpgradableReadGuard {
                rwlock: self,
                guard: g,
            })
    }

    #[inline]
    pub fn try_upgradable_read_until(
        &self,
        timeout: Instant,
    ) -> Option<RwLockUpgradableReadGuard<T>> {
        self.data
            .try_upgradable_read_until(timeout)
            .map(|g| RwLockUpgradableReadGuard {
                rwlock: self,
                guard: g,
            })
    }

    #[inline]
    pub unsafe fn force_unlock_read(&self) {
        self.data.force_unlock_read()
    }

    #[inline]
    pub unsafe fn force_unlock_write_and_save(&self) {
        self.data.force_unlock_write()
    }

    #[inline]
    pub unsafe fn force_unlock_read_fair(&self) {
        self.data.force_unlock_read_fair()
    }

    #[inline]
    pub unsafe fn force_unlock_write_and_save_fair(&self) {
        self.data.force_unlock_write_fair()
    }
}

impl<T> Debug for RwLock<T>
where
    T: Debug + Serialize + for<'de> Deserialize<'de> + ?Sized,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self.try_read() {
            Some(guard) => f
                .debug_struct("RwLock")
                .field("file_path", &self.file_path)
                .field("data", &guard)
                .finish(),
            None => {
                struct LockedPlaceholder;
                impl Debug for LockedPlaceholder {
                    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
                        f.write_str("<locked>")
                    }
                }

                f.debug_struct("RwLock")
                    .field("file_path", &self.file_path)
                    .field("data", &LockedPlaceholder)
                    .finish()
            }
        }
    }
}

pub struct RwLockReadGuard<'a, T: ?Sized> {
    rwlock: &'a RwLock<T>,
    guard: InnerRwLockReadGuard<'a, T>,
}

impl<'a, T: ?Sized> RwLockReadGuard<'a, T> {
    #[inline]
    pub fn rwlock(s: &Self) -> &'a RwLock<T> {
        s.rwlock
    }

    #[inline]
    pub fn map<U: ?Sized, F>(s: Self, f: F) -> MappedRwLockReadGuard<'a, U>
    where
        F: FnOnce(&T) -> &U,
    {
        MappedRwLockReadGuard(InnerRwLockReadGuard::map(s.guard, f))
    }

    #[inline]
    pub fn try_map<U: ?Sized, F>(s: Self, f: F) -> Result<MappedRwLockReadGuard<'a, U>, Self>
    where
        F: FnOnce(&T) -> Option<&U>,
    {
        InnerRwLockReadGuard::try_map(s.guard, f).map_or_else(
            |g| {
                Err(Self {
                    rwlock: s.rwlock,
                    guard: g,
                })
            },
            |g| Ok(MappedRwLockReadGuard(g)),
        )
    }

    #[inline]
    pub fn unlocked<F, U>(s: &mut Self, f: F) -> U
    where
        F: FnOnce() -> U,
    {
        InnerRwLockReadGuard::unlocked(&mut s.guard, f)
    }

    #[inline]
    pub fn unlocked_fair<F, U>(s: &mut Self, f: F) -> U
    where
        F: FnOnce() -> U,
    {
        InnerRwLockReadGuard::unlocked_fair(&mut s.guard, f)
    }

    #[inline]
    pub fn unlock_fair(s: Self) {
        InnerRwLockReadGuard::unlock_fair(s.guard);
    }

    #[inline]
    pub fn bump(s: &mut Self) {
        InnerRwLockReadGuard::bump(&mut s.guard);
    }
}

impl<T> Debug for RwLockReadGuard<'_, T>
where
    T: Debug + ?Sized,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Debug::fmt(&self.guard, f)
    }
}

impl<T> Display for RwLockReadGuard<'_, T>
where
    T: Display + ?Sized,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.guard.deref().fmt(f)
    }
}

impl<T: ?Sized> Deref for RwLockReadGuard<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.guard.deref()
    }
}

pub struct RwLockWriteGuard<'a, T: ?Sized> {
    rwlock: &'a RwLock<T>,
    guard: InnerRwLockWriteGuard<'a, T>,
}

impl<'a, T: ?Sized> RwLockWriteGuard<'a, T> {
    #[inline]
    pub fn rwlock(s: &Self) -> &'a RwLock<T> {
        s.rwlock
    }

    #[inline]
    pub fn map<U: ?Sized, F>(s: Self, f: F) -> MappedRwLockWriteGuard<'a, U>
    where
        F: FnOnce(&mut T) -> &mut U,
    {
        MappedRwLockWriteGuard(InnerRwLockWriteGuard::map(s.guard, f))
    }

    #[inline]
    pub fn try_map<U: ?Sized, F>(s: Self, f: F) -> Result<MappedRwLockWriteGuard<'a, U>, Self>
    where
        F: FnOnce(&mut T) -> Option<&mut U>,
    {
        InnerRwLockWriteGuard::try_map(s.guard, f).map_or_else(
            |g| {
                Err(Self {
                    rwlock: s.rwlock,
                    guard: g,
                })
            },
            |g| Ok(MappedRwLockWriteGuard(g)),
        )
    }

    #[inline]
    pub fn downgrade(s: Self) -> RwLockReadGuard<'a, T> {
        RwLockReadGuard {
            rwlock: s.rwlock,
            guard: InnerRwLockWriteGuard::downgrade(s.guard),
        }
    }

    #[inline]
    pub fn downgrade_to_upgradable(s: Self) -> RwLockUpgradableReadGuard<'a, T> {
        RwLockUpgradableReadGuard {
            rwlock: s.rwlock,
            guard: InnerRwLockWriteGuard::downgrade_to_upgradable(s.guard),
        }
    }

    #[inline]
    pub fn unlocked<F, U>(s: &mut Self, f: F) -> U
    where
        F: FnOnce() -> U,
    {
        InnerRwLockWriteGuard::unlocked(&mut s.guard, f)
    }

    #[inline]
    pub fn unlocked_fair<F, U>(s: &mut Self, f: F) -> U
    where
        F: FnOnce() -> U,
    {
        InnerRwLockWriteGuard::unlocked_fair(&mut s.guard, f)
    }

    #[inline]
    pub fn unlock_fair(s: Self) {
        InnerRwLockWriteGuard::unlock_fair(s.guard);
    }

    #[inline]
    pub fn bump(s: &mut Self) {
        InnerRwLockWriteGuard::bump(&mut s.guard);
    }
}

impl<T> Debug for RwLockWriteGuard<'_, T>
where
    T: Debug + ?Sized,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Debug::fmt(&self.guard, f)
    }
}

impl<T> Display for RwLockWriteGuard<'_, T>
where
    T: Display + ?Sized,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.guard.deref().fmt(f)
    }
}

impl<T: ?Sized + Serialize> Deref for RwLockWriteGuard<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.guard.deref()
    }
}

impl<T: ?Sized + Serialize> DerefMut for RwLockWriteGuard<'_, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        self.guard.deref_mut()
    }
}

pub struct RwLockUpgradableReadGuard<'a, T: ?Sized> {
    rwlock: &'a RwLock<T>,
    guard: InnerRwLockUpgradableReadGuard<'a, T>,
}

impl<'a, T: ?Sized> RwLockUpgradableReadGuard<'a, T> {
    #[inline]
    pub fn rwlock(s: &Self) -> &'a RwLock<T> {
        s.rwlock
    }

    #[inline]
    pub fn downgrade(s: Self) -> RwLockReadGuard<'a, T> {
        RwLockReadGuard {
            rwlock: s.rwlock,
            guard: InnerRwLockUpgradableReadGuard::downgrade(s.guard),
        }
    }

    #[inline]
    pub fn upgrade(s: Self) -> RwLockWriteGuard<'a, T> {
        RwLockWriteGuard {
            rwlock: s.rwlock,
            guard: InnerRwLockUpgradableReadGuard::upgrade(s.guard),
        }
    }

    #[inline]
    pub fn try_upgrade(s: Self) -> Result<RwLockWriteGuard<'a, T>, Self> {
        InnerRwLockUpgradableReadGuard::try_upgrade(s.guard).map_or_else(
            |g| {
                Err(RwLockUpgradableReadGuard {
                    rwlock: s.rwlock,
                    guard: g,
                })
            },
            |g| {
                Ok(RwLockWriteGuard {
                    rwlock: s.rwlock,
                    guard: g,
                })
            },
        )
    }

    #[inline]
    pub fn try_upgrade_for(s: Self, timeout: Duration) -> Result<RwLockWriteGuard<'a, T>, Self> {
        InnerRwLockUpgradableReadGuard::try_upgrade_for(s.guard, timeout).map_or_else(
            |g| {
                Err(RwLockUpgradableReadGuard {
                    rwlock: s.rwlock,
                    guard: g,
                })
            },
            |g| {
                Ok(RwLockWriteGuard {
                    rwlock: s.rwlock,
                    guard: g,
                })
            },
        )
    }

    #[inline]
    pub fn try_upgrade_until(s: Self, timeout: Instant) -> Result<RwLockWriteGuard<'a, T>, Self> {
        InnerRwLockUpgradableReadGuard::try_upgrade_until(s.guard, timeout).map_or_else(
            |g| {
                Err(RwLockUpgradableReadGuard {
                    rwlock: s.rwlock,
                    guard: g,
                })
            },
            |g| {
                Ok(RwLockWriteGuard {
                    rwlock: s.rwlock,
                    guard: g,
                })
            },
        )
    }

    #[inline]
    pub fn unlocked<F, U>(s: &mut Self, f: F) -> U
    where
        F: FnOnce() -> U,
    {
        InnerRwLockUpgradableReadGuard::unlocked(&mut s.guard, f)
    }

    #[inline]
    pub fn unlocked_fair<F, U>(s: &mut Self, f: F) -> U
    where
        F: FnOnce() -> U,
    {
        InnerRwLockUpgradableReadGuard::unlocked_fair(&mut s.guard, f)
    }

    #[inline]
    pub fn unlock_fair(s: Self) {
        InnerRwLockUpgradableReadGuard::unlock_fair(s.guard);
    }

    #[inline]
    pub fn bump(s: &mut Self) {
        InnerRwLockUpgradableReadGuard::bump(&mut s.guard);
    }
}

impl<T> Debug for RwLockUpgradableReadGuard<'_, T>
where
    T: Debug + ?Sized,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Debug::fmt(&self.guard, f)
    }
}

impl<T> Display for RwLockUpgradableReadGuard<'_, T>
where
    T: Display + ?Sized,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.guard.deref().fmt(f)
    }
}

impl<T: ?Sized> Deref for RwLockUpgradableReadGuard<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.guard.deref()
    }
}

pub struct MappedRwLockReadGuard<'a, T: ?Sized>(InnerMappedRwLockReadGuard<'a, T>);

impl<'a, T: ?Sized> MappedRwLockReadGuard<'a, T> {
    #[inline]
    pub fn map<U: ?Sized, F>(s: Self, f: F) -> MappedRwLockReadGuard<'a, U>
    where
        F: FnOnce(&T) -> &U,
    {
        MappedRwLockReadGuard(InnerMappedRwLockReadGuard::map(s.0, f))
    }

    #[inline]
    pub fn try_map<U: ?Sized, F>(s: Self, f: F) -> Result<MappedRwLockReadGuard<'a, U>, Self>
    where
        F: FnOnce(&T) -> Option<&U>,
    {
        InnerMappedRwLockReadGuard::try_map(s.0, f)
            .map_or_else(|g| Err(Self(g)), |g| Ok(MappedRwLockReadGuard(g)))
    }

    #[inline]
    pub fn unlock_fair(s: Self) {
        InnerMappedRwLockReadGuard::unlock_fair(s.0);
    }
}

impl<T> Debug for MappedRwLockReadGuard<'_, T>
where
    T: Debug + ?Sized,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Debug::fmt(&self.0, f)
    }
}

impl<T> Display for MappedRwLockReadGuard<'_, T>
where
    T: Display + ?Sized,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.0.deref().fmt(f)
    }
}

impl<T: ?Sized> Deref for MappedRwLockReadGuard<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

pub struct MappedRwLockWriteGuard<'a, T: ?Sized>(InnerMappedRwLockWriteGuard<'a, T>);

impl<'a, T: ?Sized> MappedRwLockWriteGuard<'a, T> {
    #[inline]
    pub fn map<U: ?Sized, F>(s: Self, f: F) -> MappedRwLockWriteGuard<'a, U>
    where
        F: FnOnce(&mut T) -> &mut U,
    {
        MappedRwLockWriteGuard(InnerMappedRwLockWriteGuard::map(s.0, f))
    }

    #[inline]
    pub fn try_map<U: ?Sized, F>(s: Self, f: F) -> Result<MappedRwLockWriteGuard<'a, U>, Self>
    where
        F: FnOnce(&mut T) -> Option<&mut U>,
    {
        InnerMappedRwLockWriteGuard::try_map(s.0, f)
            .map_or_else(|g| Err(Self(g)), |g| Ok(MappedRwLockWriteGuard(g)))
    }

    #[inline]
    pub fn unlock_fair(s: Self) {
        InnerMappedRwLockWriteGuard::unlock_fair(s.0);
    }
}

impl<T> Debug for MappedRwLockWriteGuard<'_, T>
where
    T: Debug + ?Sized,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Debug::fmt(&self.0, f)
    }
}

impl<T> Display for MappedRwLockWriteGuard<'_, T>
where
    T: Display + ?Sized,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.0.deref().fmt(f)
    }
}

impl<T: ?Sized + Serialize> Deref for MappedRwLockWriteGuard<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl<T: ?Sized + Serialize> DerefMut for MappedRwLockWriteGuard<'_, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        self.0.deref_mut()
    }
}
