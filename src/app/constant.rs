pub mod header;

/// 533 Upstream Failure
/// [A non-standard code. Indicates the server, while acting as a gateway or proxy,
/// received a response from an upstream service that constituted a failure. Unlike
/// 502 (Bad Gateway), which implies an invalid or unparseable response, this code
/// suggests the upstream service itself reported an error (e.g., returned a 5xx status).]
pub const UPSTREAM_FAILURE: http::StatusCode = unsafe { core::intrinsics::transmute(533u16) };

#[macro_export]
macro_rules! def_pub_const {
    // 单个常量定义
    // ($name:ident, $value:expr) => {
    //     pub const $name: &'static str = $value;
    // };

    // 批量常量定义
    ($($(#[$meta:meta])* $name:ident = $value:expr),+ $(,)?) => {
        $(
            $(#[$meta])*
            pub const $name: &'static str = $value;
        )+
    };
}

#[macro_export]
macro_rules! define_typed_constants {
    // 递归情况：处理一个类型块，然后继续处理剩余的
    (
        $vis:vis $ty:ty => {
            $(
                $(#[$attr:meta])*
                $name:ident = $value:expr
            ),* $(,)?
        }
        $($rest:tt)*
    ) => {
        $(
            $(#[$attr])*
            $vis const $name: $ty = $value;
        )*

        // 递归处理剩余的类型块
        $crate::define_typed_constants! {
            $($rest)*
        }
    };

    // 基础情况：没有更多内容时停止
    () => {};
}

pub const COMMA: char = ',';

#[cfg(feature = "__preview")]
pub use crate::common::build::BUILD_VERSION;
pub use crate::common::build::{BUILD_TIMESTAMP, IS_DEBUG, IS_PRERELEASE, VERSION};

pub const MIN_COMPAT_VERSION: super::model::version::Version =
    super::model::version::preview(0, 4, 0, 23);

pub struct ExeName(bool);

impl ExeName {
    #[cfg(windows)]
    pub const EXE_NAME: &'static str = concat!(env!("CARGO_PKG_NAME"), ".exe");
    #[cfg(not(windows))]
    pub const EXE_NAME: &'static str = PKG_NAME;

    pub const YELLOW: Self = Self(false);
    pub const BRIGHT_RED: Self = Self(true);
}

impl core::fmt::Display for ExeName {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        const BOLD_YELLOW: &str = "\x1b[1;33m"; // 粗体 + 黄色
        const BOLD_BRIGHT_RED: &str = "\x1b[1;91m"; // 粗体 + 亮红色
        const RESET: &str = "\x1b[0m"; // 重置所有样式
        if unsafe { IS_TERMINAL } {
            f.write_str(if self.0 { BOLD_BRIGHT_RED } else { BOLD_YELLOW })?;
            f.write_str(Self::EXE_NAME)?;
            f.write_str(RESET)
        } else {
            f.write_str(Self::EXE_NAME)
        }
    }
}

// Package related constants
def_pub_const!(
    PKG_VERSION = env!("CARGO_PKG_VERSION"),
    PKG_NAME = env!("CARGO_PKG_NAME"),
    // PKG_DESCRIPTION = env!("CARGO_PKG_DESCRIPTION"),
    // PKG_AUTHORS = env!("CARGO_PKG_AUTHORS"),
    // PKG_REPOSITORY = env!("CARGO_PKG_REPOSITORY")
);

// Basic string constants
def_pub_const!(
    EMPTY_STRING = "",
    // COMMA_STRING = ",",
    UNKNOWN = "unknown",
    TYPE = "type",
    ERROR = "error"
);

// Route related constants
def_pub_const!(
    // ROUTE_ROOT_PATH = "/",
    ROUTE_HEALTH_PATH = "/health",
    ROUTE_GEN_UUID_PATH = "/gen-uuid",
    ROUTE_GEN_HASH_PATH = "/gen-hash",
    ROUTE_GEN_CHECKSUM_PATH = "/gen-checksum",
    ROUTE_GET_CHECKSUM_HEADER_PATH = "/get-checksum-header",
    // ROUTE_USER_INFO_PATH = "/userinfo",
    // ROUTE_API_PATH = "/api",
    // ROUTE_LOGS_PATH = "/logs",
    ROUTE_LOGS_GET_PATH = "/logs/get",
    ROUTE_LOGS_TOKENS_GET_PATH = "/logs/tokens/get",
    // ROUTE_CONFIG_PATH = "/config",
    ROUTE_CONFIG_GET_PATH = "/config/get",
    ROUTE_CONFIG_SET_PATH = "/config/set",
    ROUTE_CONFIG_RELOAD_PATH = "/config/reload",
    // ROUTE_TOKENS_PATH = "/tokens",
    ROUTE_TOKENS_GET_PATH = "/tokens/get",
    ROUTE_TOKENS_SET_PATH = "/tokens/set",
    ROUTE_TOKENS_ADD_PATH = "/tokens/add",
    ROUTE_TOKENS_DELETE_PATH = "/tokens/del",
    ROUTE_TOKENS_ALIAS_SET_PATH = "/tokens/alias/set",
    ROUTE_TOKENS_PROFILE_UPDATE_PATH = "/tokens/profile/update",
    ROUTE_TOKENS_CONFIG_VERSION_UPDATE_PATH = "/tokens/config-version/update",
    ROUTE_TOKENS_REFRESH_PATH = "/tokens/refresh",
    ROUTE_TOKENS_STATUS_SET_PATH = "/tokens/status/set",
    ROUTE_TOKENS_PROXY_SET_PATH = "/tokens/proxy/set",
    ROUTE_TOKENS_TIMEZONE_SET_PATH = "/tokens/timezone/set",
    ROUTE_TOKENS_MERGE_PATH = "/tokens/merge",
    // ROUTE_PROXIES_PATH = "/proxies",
    ROUTE_PROXIES_GET_PATH = "/proxies/get",
    ROUTE_PROXIES_SET_PATH = "/proxies/set",
    ROUTE_PROXIES_ADD_PATH = "/proxies/add",
    ROUTE_PROXIES_DELETE_PATH = "/proxies/del",
    ROUTE_PROXIES_SET_GENERAL_PATH = "/proxies/set-general",
    ROUTE_NTP_SYNC_ONCE_PATH = "/ntp/sync-once",
    ROUTE_ENV_EXAMPLE_PATH = "/env-example",
    ROUTE_CONFIG_EXAMPLE_PATH = "/config-example",
    // ROUTE_STATIC_PATH = "/static/{path}",
    // ROUTE_SHARED_STYLES_PATH = "/static/shared-styles.css",
    // ROUTE_SHARED_JS_PATH = "/static/shared.js",
    // ROUTE_ABOUT_PATH = "/about",
    ROUTE_LICENSE_PATH = "/license",
    ROUTE_README_PATH = "/readme",
    ROUTE_BUILD_KEY_PATH = "/build-key",
    ROUTE_CONFIG_VERSION_GET_PATH = "/config-version/get",
    ROUTE_TOKEN_PROFILE_GET_PATH = "/token-profile/get",
    ROUTE_CPP_CONFIG_PATH = "/cpp/config",
    ROUTE_CPP_MODELS_PATH = "/cpp/models",
    ROUTE_FILE_UPLOAD_PATH = "/file/upload",
    ROUTE_FILE_SYNC_PATH = "/file/sync",
    ROUTE_CPP_STREAM_PATH = "/cpp/stream",
    ROUTE_RAW_MODELS_PATH = "/raw/models",
    ROUTE_MODELS_PATH = "/v1/models",
    ROUTE_CHAT_COMPLETIONS_PATH = "/v1/chat/completions",
    ROUTE_CLI_CHAT_COMPLETIONS_PATH = "/cli/chat/completions",
    ROUTE_MESSAGES_PATH = "/v1/messages",
    ROUTE_MESSAGES_COUNT_TOKENS_PATH = "/v1/messages/count_tokens",
);

// Status constants
def_pub_const!(STATUS_PENDING = "pending", STATUS_SUCCESS = "success", STATUS_FAILURE = "failure");

// Authorization constants
def_pub_const!(AUTHORIZATION_BEARER_PREFIX = "Bearer ");

// Cursor related constants
def_pub_const!(
    CURSOR_API2_HOST = "api2.cursor.sh",
    CURSOR_HOST = "cursor.com",
    CURSOR_API4_HOST = "api4.cursor.sh",
    CURSOR_GCPP_ASIA_HOST = "us-asia.gcpp.cursor.sh",
    CURSOR_GCPP_EU_HOST = "us-eu.gcpp.cursor.sh",
    CURSOR_GCPP_US_HOST = "us-only.gcpp.cursor.sh"
);

// Object type constants
def_pub_const!(
    CHATCMPL_PREFIX = "chatcmpl-",
    MSG01_PREFIX = "msg_01",
    // TOOLU01_PREFIX = "toolu_01",
    // OBJECT_TEXT_COMPLETION = "text_completion"
);

// def_pub_const!(
//     CURSOR_API2_STREAM_CHAT = "StreamChat",
//     CURSOR_API2_GET_USER_INFO = "GetUserInfo"
// );

// Error message constants
def_pub_const!(
    ERR_STREAM_RESPONSE = "Empty stream response",
    ERR_RESPONSE_RECEIVED = "Empty response received",
    ERR_LOG_TOKEN_NOT_FOUND = "日志对应的token必须存在 - 数据一致性错误",
    // INVALID_STREAM = "invalid_stream"
);

// def_pub_const!(ERR_CHECKSUM_NO_GOOD = "checksum no good");

def_pub_const!(
    HEADER_B64 = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.",
    ISSUER = "https://authentication.cursor.sh",
    SCOPE = "openid profile email offline_access",
    AUDIENCE = "https://cursor.com",
    TYPE_SESSION = "session",
    TYPE_WEB = "web"
);

def_pub_const!(ASIA = "Asia", EU = "EU", US = "US");

def_pub_const!(
    UNNAMED = unsafe { core::str::from_raw_parts(UNNAMED_PATTERN.as_ptr(), 7) },
    UNNAMED_PATTERN = "unnamed-"
);

def_pub_const!(HTTPS_PREFIX = "https://");

// def_pub_const! {
//     DEFAULT_THINKING_TAG = unsafe { core::str::from_raw_parts(DEFAULT_THINKING_TAG_OPEN.as_ptr().add(1), 5) },
//     DEFAULT_THINKING_TAG_OPEN = "<think>",
//     DEFAULT_THINKING_TAG_CLOSE = "</think>"
// }

// pub static THINKING_TAG_OPEN: ManuallyInit<&'static str> =
//     ManuallyInit::new_with(DEFAULT_THINKING_TAG_OPEN);
// pub static THINKING_TAG_CLOSE: ManuallyInit<&'static str> =
//     ManuallyInit::new_with(DEFAULT_THINKING_TAG_CLOSE);

// #[deny(unused)]
// pub fn init_thinking_tags() {
//     unsafe {
//         let tag = crate::common::utils::parse_from_env("THINKING_TAG", DEFAULT_THINKING_TAG);

//         if tag == DEFAULT_THINKING_TAG {
//             return;
//         }

//         // 检查标签长度限制
//         const MAX_TAG_LEN: usize = 16;
//         let tag_len = tag.len();
//         if tag_len > MAX_TAG_LEN - 3 {
//             __eprintln!("Warning: THINKING_TAG too long, using default");
//             return;
//         }

//         let mut buf = [core::mem::MaybeUninit::<u8>::uninit(); MAX_TAG_LEN];
//         let tag_bytes = tag.as_bytes();

//         // 构建开始标签 <tag>
//         buf[1].write(b'<');
//         ::core::ptr::copy_nonoverlapping(
//             tag_bytes.as_ptr(),
//             buf.as_mut_ptr().add(2).cast(),
//             tag_len,
//         );
//         let open_len = tag_len + 2;
//         *buf.get_unchecked_mut(open_len) = core::mem::MaybeUninit::new(b'>');

//         // 分配开始标签
//         let open_layout = ::core::alloc::Layout::from_size_align_unchecked(open_len, 1);
//         let open_ptr = alloc::alloc::alloc(open_layout);
//         if open_ptr.is_null() {
//             alloc::alloc::handle_alloc_error(open_layout);
//         }
//         ::core::ptr::copy_nonoverlapping(buf.as_ptr().add(1).cast(), open_ptr, open_len);
//         THINKING_TAG_OPEN.init(::core::str::from_utf8_unchecked(::core::slice::from_raw_parts(
//             open_ptr, open_len,
//         )));

//         // 构建结束标签 </tag>
//         buf[0].write(b'<');
//         buf[1].write(b'/');
//         let close_len = open_len + 1;

//         // 分配结束标签
//         let close_layout = ::core::alloc::Layout::from_size_align_unchecked(close_len, 1);
//         let close_ptr = alloc::alloc::alloc(close_layout);
//         if close_ptr.is_null() {
//             alloc::alloc::handle_alloc_error(close_layout);
//         }
//         ::core::ptr::copy_nonoverlapping(buf.as_ptr().cast(), close_ptr, close_len);
//         THINKING_TAG_CLOSE.init(::core::str::from_utf8_unchecked(::core::slice::from_raw_parts(
//             close_ptr, close_len,
//         )));
//     }
// }

pub static mut IS_TERMINAL: bool = false;
