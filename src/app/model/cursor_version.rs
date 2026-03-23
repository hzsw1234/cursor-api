use alloc::sync::Arc;
use arc_swap::ArcSwap;
use core::{
    fmt::{self, Display, Formatter},
    num::ParseIntError,
    str::FromStr,
};
use manually_init::ManuallyInit;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Version {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl Version {
    pub fn to_bytes(self) -> [u8; 6] {
        unsafe { core::mem::transmute([self.major, self.minor, self.patch]) }
    }
}

impl Default for Version {
    fn default() -> Self { Self { major: 2, minor: 6, patch: 0 } }
}

impl<'de> serde::Deserialize<'de> for Version {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl FromStr for Version {
    type Err = ParseIntError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = [0; 3];
        for (s, i) in s.split('.').zip(parts.iter_mut()) {
            *i = s.parse()?;
        }
        Ok(Version { major: parts[0], minor: parts[1], patch: parts[2] })
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut buffer = itoa::Buffer::new();
        f.write_str(buffer.format(self.major))?;
        f.write_str(".")?;
        f.write_str(buffer.format(self.minor))?;
        f.write_str(".")?;
        f.write_str(buffer.format(self.patch))?;
        Ok(())
    }
}

struct VersionValues {
    /// 客户端版本的 HeaderValue
    client_version: http::header::HeaderValue,
    /// Cursor User-Agent 的 HeaderValue
    ua_cursor: http::header::HeaderValue,
}

static INSTANCE: ManuallyInit<ArcSwap<VersionValues>> = ManuallyInit::new();

pub fn get() -> bytes::Bytes {
    use crate::common::model::HeaderValue;
    let v: &HeaderValue = (&INSTANCE.load().client_version).into();
    v.inner.clone()
}

pub fn client_version() -> http::header::HeaderValue { INSTANCE.load().client_version.clone() }

pub fn ua_cursor() -> http::header::HeaderValue { INSTANCE.load().ua_cursor.clone() }

pub(super) fn init(platform: super::platform::PlatformType, version: Version) {
    use crate::common::model::HeaderValue;
    let version = version.to_string();
    INSTANCE.init(ArcSwap::from_pointee(unsafe {
        VersionValues {
            ua_cursor: HeaderValue::from(platform.as_platform().client_ua(&version)).into(),
            client_version: HeaderValue::from(version).into(),
        }
    }))
}

pub(super) fn update(platform: super::platform::PlatformType, version: Version) {
    use crate::common::model::HeaderValue;
    let version = version.to_string();
    INSTANCE.store(Arc::new(unsafe {
        VersionValues {
            ua_cursor: HeaderValue::from(platform.as_platform().client_ua(&version)).into(),
            client_version: HeaderValue::from(version).into(),
        }
    }))
}

pub(super) fn update_platform_only(platform: super::platform::PlatformType) {
    use crate::common::model::HeaderValue;
    let guard = INSTANCE.load();
    let version = unsafe { core::str::from_utf8_unchecked(guard.client_version.as_bytes()) };
    INSTANCE.store(Arc::new(unsafe {
        VersionValues {
            ua_cursor: HeaderValue::from(platform.as_platform().client_ua(version)).into(),
            client_version: guard.client_version.clone(),
        }
    }))
}
