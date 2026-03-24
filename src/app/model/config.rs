use super::{FetchMode, UsageCheck, VisionAbility};
use crate::app::{
    lazy::CONFIG_FILE_PATH,
    model::{Hash, cursor_version::Version, platform::PlatformType},
};
use arc_swap::ArcSwap;
use byte_str::ByteStr;
use manually_init::ManuallyInit;
use serde::Deserialize;

// 静态配置
#[derive(Debug, Default, Clone, Deserialize)]
pub struct AppConfig {
    pub vision_ability: VisionAbility,
    pub slow_pool_enabled: bool,
    pub long_context_enabled: bool,
    pub model_usage_checks: UsageCheck,
    pub dynamic_key_secret: String,
    pub share_token: String,
    pub web_references_included: bool,
    pub raw_model_fetch_mode: FetchMode,
    pub emulated_platform: PlatformType,
    pub cursor_client_version: Version,
}

pub struct AppConfigWrapper {
    pub hash: Hash,
    pub inner: AppConfig,
    pub content: ByteStr,
}

impl core::ops::Deref for AppConfigWrapper {
    type Target = AppConfig;
    fn deref(&self) -> &Self::Target { &self.inner }
}

// 全局配置实例
static APP_CONFIG: ManuallyInit<ArcSwap<AppConfigWrapper>> = ManuallyInit::new();

macro_rules! config_methods {
    // 递归终止
    () => {};

    // 语法: 字段名: 类型 as 新方法名;
    ($field:ident: $type:ty as $method:ident; $($rest:tt)*) => {
        #[inline]
        pub fn $method() -> $type {
            APP_CONFIG.load().$field.clone()
        }
        config_methods!($($rest)*);
    };

    // 语法: 字段名: 类型;
    ($field:ident: $type:ty; $($rest:tt)*) => {
        #[inline]
        pub fn $field() -> $type {
            APP_CONFIG.load().$field.clone()
        }
        config_methods!($($rest)*);
    };
}

impl AppConfig {
    pub fn init() {
        // base
        {
            interned::init();
            super::token::__init()
        };
        // env
        {
            super::super::lazy::init();
            crate::common::model::ntp::Servers::init();
            super::tz::__init();
            super::super::lazy::init_all_cursor_urls();
            super::super::lazy::log::init();
            super::hash::init();
            // super::super::constant::header::initialize_cursor_version();
            // super::super::constant::init_thinking_tags();
            crate::core::model::init_resolver();
            super::token::parse_providers();
            super::context_fill_mode::init();
        }
        crate::core::constant::create_models();
        crate::core::service::cli_backend::init();

        let (content, config) = if let Ok(s) = std::fs::read_to_string(&*CONFIG_FILE_PATH) {
            match toml::from_str(&s) {
                Ok(config) => (s.into(), config),
                Err(e) => {
                    eprintln!("Warning: configuration parse failed: {e}");
                    (ByteStr::new(), AppConfig::default())
                }
            }
        } else {
            (ByteStr::new(), AppConfig::default())
        };
        {
            use super::cursor_version::init;
            init(config.emulated_platform, config.cursor_client_version)
        }
        {
            use super::dynamic_key::{Secret, init};
            init(Secret::parse_str(&config.dynamic_key_secret).0.unwrap_or([0; 64]));
        }
        APP_CONFIG.init(ArcSwap::from_pointee(AppConfigWrapper {
            hash: hash(&config),
            inner: config,
            content,
        }));
    }

    pub fn update(config: Self, content: ByteStr) -> bool {
        let hash = hash(&config);
        let guard = APP_CONFIG.load();
        if guard.hash != hash {
            {
                use super::cursor_version::{update, update_platform_only};
                if guard.cursor_client_version != config.cursor_client_version {
                    update(config.emulated_platform, config.cursor_client_version)
                } else if guard.emulated_platform != config.emulated_platform {
                    update_platform_only(config.emulated_platform)
                }
            }
            if guard.dynamic_key_secret != config.dynamic_key_secret {
                use super::dynamic_key::{Secret, update};
                update(Secret::parse_str(&config.dynamic_key_secret).0.unwrap_or([0; 64]));
            }
            APP_CONFIG.store(alloc::sync::Arc::new(AppConfigWrapper {
                hash,
                inner: config,
                content,
            }));
            true
        } else {
            false
        }
    }

    pub fn hash() -> Hash { APP_CONFIG.load().hash }

    pub fn content() -> (Hash, ByteStr) {
        let config = APP_CONFIG.load();
        (config.hash, config.content.clone())
    }

    config_methods!(
        vision_ability: VisionAbility;
        slow_pool_enabled: bool as is_slow_pool_enabled;
        long_context_enabled: bool as is_long_context_enabled;
        model_usage_checks: UsageCheck;
        // dynamic_key_secret: Str;
        // share_token: Str;
        web_references_included: bool as is_web_references_included;
        raw_model_fetch_mode: FetchMode;
        emulated_platform: PlatformType;
    );

    #[inline]
    pub fn is_dynamic_key_enabled() -> bool { !APP_CONFIG.load().dynamic_key_secret.is_empty() }

    #[inline]
    pub fn share_token_eq(s: &str) -> bool { APP_CONFIG.load().share_token == s }

    #[inline]
    pub fn is_share() -> bool { APP_CONFIG.load().share_token.is_empty() }
}

fn hash(config: &AppConfig) -> Hash {
    use sha2::{Digest as _, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(b"vision_ability");
    hasher.update(config.vision_ability.as_str().as_bytes());
    hasher.update(b"slow_pool_enabled");
    hasher.update([config.slow_pool_enabled as u8]);
    hasher.update(b"long_context_enabled");
    hasher.update([config.long_context_enabled as u8]);
    hasher.update(b"model_usage_checks");
    config.model_usage_checks.hash(&mut hasher);
    hasher.update(b"dynamic_key_secret");
    hasher.update(config.dynamic_key_secret.as_bytes());
    hasher.update(b"share_token");
    hasher.update(config.share_token.as_bytes());
    hasher.update(b"web_references_included");
    hasher.update([config.web_references_included as u8]);
    hasher.update(b"raw_model_fetch_mode");
    hasher.update(config.raw_model_fetch_mode.as_str().as_bytes());
    hasher.update(b"emulated_platform");
    hasher.update(config.emulated_platform.as_str().as_bytes());
    hasher.update(b"cursor_client_version");
    hasher.update(config.cursor_client_version.to_bytes());
    Hash(hasher.finalize().0)
}

#[test]
fn test() { println!("{}", hash(&AppConfig::default())) }
