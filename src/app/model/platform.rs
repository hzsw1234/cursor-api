use crate::common::{model::HeaderValue, platform::DEFAULT};

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformType {
    Windows,
    macOS,
    Linux,
}

impl Default for PlatformType {
    fn default() -> Self { DEFAULT }
}

impl PlatformType {
    const WINDOWS: &str = "Windows";
    const MAC_OS: &str = "macOS";
    const LINUX: &str = "Linux";
    pub const fn as_str(self) -> &'static str {
        match self {
            PlatformType::Windows => Self::WINDOWS,
            PlatformType::macOS => Self::MAC_OS,
            PlatformType::Linux => Self::LINUX,
        }
    }
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            Self::WINDOWS => Some(PlatformType::Windows),
            Self::MAC_OS => Some(PlatformType::macOS),
            Self::LINUX => Some(PlatformType::Linux),
            _ => None,
        }
    }
}

impl serde::Serialize for PlatformType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer {
        self.as_str().serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for PlatformType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s)
            .ok_or_else(|| serde::de::Error::custom(format_args!("{s:?} is unsupported")))
    }
}

pub struct Platforms {
    pub windows: Platform,
    pub macos: Platform,
    pub linux: Platform,
}

pub struct Platform {
    pub web_ua: &'static str,
    pub string: &'static str,
    pub ua_prefix: &'static str,
    // pub default_ua: &'static str,
}

impl PlatformType {
    pub const fn as_platform(self) -> &'static Platform {
        match self {
            PlatformType::Windows => &PLATFORMS.windows,
            PlatformType::macOS => &PLATFORMS.macos,
            PlatformType::Linux => &PLATFORMS.linux,
        }
    }
}

/// User-Agent 后缀
pub const UA_SUFFIX: &'static str = " Chrome/140.0.7339.41 Electron/38.0.0 Safari/537.36";

// const UA_SUFFIX_LEN: usize = " Chrome/140.0.7339.41 Electron/38.0.0 Safari/537.36".len();

static PLATFORMS: Platforms = Platforms {
    windows: Platform {
        web_ua: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/145.0.0.0 Safari/537.36",
        string: "\"Windows\"",
        ua_prefix: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Cursor/",
        // default_ua: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Cursor/2.6.0 Chrome/140.0.7339.41 Electron/38.0.0 Safari/537.36",
    },
    macos: Platform {
        web_ua: "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/145.0.0.0 Safari/537.36",
        string: "\"macOS\"",
        ua_prefix: "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Cursor/",
        // default_ua: "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Cursor/2.6.0 Chrome/140.0.7339.41 Electron/38.0.0 Safari/537.36",
    },
    linux: Platform {
        web_ua: "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/145.0.0.0 Safari/537.36",
        string: "\"Linux\"",
        ua_prefix: "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Cursor/",
        // default_ua: "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Cursor/2.6.0 Chrome/140.0.7339.41 Electron/38.0.0 Safari/537.36",
    },
};

impl Platform {
    pub const fn web_ua(&self) -> http::header::HeaderValue {
        unsafe { HeaderValue::from_static(self.web_ua).into() }
    }
    // pub const fn as_str(&self) -> &'static str {
    //     let ptr = self.string.as_ptr();
    //     let len = core::ptr::metadata(self.string);
    //     unsafe { &*core::ptr::from_raw_parts(ptr.add(1), len.unchecked_sub(2)) }
    // }
    pub const fn as_header_value(&self) -> http::header::HeaderValue {
        unsafe { HeaderValue::from_static(self.string).into() }
    }
    /// User-Agent 前缀
    pub const fn ua_prefix(&self) -> &'static str { self.ua_prefix }
    // const fn ua_suffix(&self) -> &'static str {
    //     use core::slice::SliceIndex as _;
    //     unsafe {
    //         &*(self.default_ua.len().unchecked_sub(UA_SUFFIX_LEN)..).get_unchecked(self.default_ua)
    //     }
    // }
    // /// 默认的 User-Agent
    // pub const fn default_ua(&self) -> &'static str { self.default_ua }
    pub fn client_ua(&self, version: &str) -> String {
        [self.ua_prefix(), version, UA_SUFFIX].concat()
    }
}

pub fn current() -> &'static Platform { super::AppConfig::emulated_platform().as_platform() }
