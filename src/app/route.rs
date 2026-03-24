mod json;

pub use json::{InfallibleSerialize, InfallibleJson, GenericJson, OpenAiJson, AnthropicJson};

use super::{
    constant::{
        ROUTE_BUILD_KEY_PATH, ROUTE_CHAT_COMPLETIONS_PATH, ROUTE_CLI_CHAT_COMPLETIONS_PATH,
        ROUTE_CONFIG_EXAMPLE_PATH,
        ROUTE_CONFIG_GET_PATH, ROUTE_CONFIG_RELOAD_PATH, ROUTE_CONFIG_SET_PATH,
        ROUTE_CONFIG_VERSION_GET_PATH, ROUTE_CPP_CONFIG_PATH, ROUTE_CPP_MODELS_PATH,
        ROUTE_CPP_STREAM_PATH, ROUTE_ENV_EXAMPLE_PATH, ROUTE_FILE_SYNC_PATH,
        ROUTE_FILE_UPLOAD_PATH, ROUTE_GEN_CHECKSUM_PATH, ROUTE_GEN_HASH_PATH, ROUTE_GEN_UUID_PATH,
        ROUTE_GET_CHECKSUM_HEADER_PATH, ROUTE_HEALTH_PATH, ROUTE_LICENSE_PATH, ROUTE_LOGS_GET_PATH,
        ROUTE_LOGS_TOKENS_GET_PATH, ROUTE_MESSAGES_COUNT_TOKENS_PATH, ROUTE_MESSAGES_PATH,
        ROUTE_MODELS_PATH, ROUTE_NTP_SYNC_ONCE_PATH, ROUTE_PROXIES_ADD_PATH,
        ROUTE_PROXIES_DELETE_PATH, ROUTE_PROXIES_GET_PATH, ROUTE_PROXIES_SET_GENERAL_PATH,
        ROUTE_PROXIES_SET_PATH, ROUTE_RAW_MODELS_PATH, ROUTE_README_PATH,
        ROUTE_TOKEN_PROFILE_GET_PATH, ROUTE_TOKENS_ADD_PATH, ROUTE_TOKENS_ALIAS_SET_PATH,
        ROUTE_TOKENS_CONFIG_VERSION_UPDATE_PATH, ROUTE_TOKENS_DELETE_PATH, ROUTE_TOKENS_GET_PATH,
        ROUTE_TOKENS_MERGE_PATH, ROUTE_TOKENS_PROFILE_UPDATE_PATH, ROUTE_TOKENS_PROXY_SET_PATH,
        ROUTE_TOKENS_REFRESH_PATH, ROUTE_TOKENS_SET_PATH, ROUTE_TOKENS_STATUS_SET_PATH,
        ROUTE_TOKENS_TIMEZONE_SET_PATH,
    },
    model::AppState,
};
use crate::{
    common::utils::parse_from_env,
    core::{
        auth::{
            admin_auth_middleware, cpp_auth_middleware, v1_auth_middleware, v1_auth2_middleware,
        },
        route::{
            handle_add_proxy, handle_add_tokens, handle_build_key, handle_config_example,
            handle_delete_proxies, handle_delete_tokens, handle_env_example, handle_gen_checksum,
            handle_gen_hash, handle_gen_uuid, handle_get_checksum_header, handle_get_config,
            handle_get_config_version, handle_get_logs, handle_get_logs_tokens, handle_get_proxies,
            handle_get_token_profile, handle_get_tokens, handle_health, handle_license,
            handle_merge_tokens, handle_ntp_sync_once, handle_readme, handle_refresh_tokens,
            handle_reload_config, handle_set_config, handle_set_general_proxy, handle_set_proxies,
            handle_set_tokens, handle_set_tokens_alias, handle_set_tokens_proxy,
            handle_set_tokens_status, handle_set_tokens_timezone,
            handle_update_tokens_config_version, handle_update_tokens_profile,
        },
        service::{
            cli_backend::handle_cli_chat_completions,
            cpp::{
                handle_cpp_config, handle_cpp_models, handle_stream_cpp, handle_sync_file,
                handle_upload_file,
            },
            handle_chat_completions, handle_messages, handle_messages_count_tokens, handle_models,
            handle_raw_models,
        },
    },
};
use alloc::sync::Arc;
use axum::{
    Router, middleware,
    routing::{get, post},
};
use tower_http::{cors::CorsLayer, limit::RequestBodyLimitLayer};

pub fn create_router(state: Arc<AppState>) -> Router {
    let (routes, mut exchange_map) = match super::frontend::init_frontend() {
        Ok(result) => result,
        Err(e) => {
            eprintln!("{e}");
            Default::default()
        }
    };

    let mut backend = Router::new()
        .without_v07_checks()
        // .route(exchange_map.resolve(ROUTE_ROOT_PATH), get(handle_root))
        .route(exchange_map.resolve(ROUTE_HEALTH_PATH), get(handle_health))
        // .route(exchange_map.resolve(ROUTE_TOKENS_PATH), get(handle_tokens_page))
        // .route(exchange_map.resolve(ROUTE_PROXIES_PATH), get(handle_proxies_page))
        .merge(
            Router::new()
                .without_v07_checks()
                .route(exchange_map.resolve(ROUTE_CONFIG_GET_PATH), post(handle_get_config))
                .route(exchange_map.resolve(ROUTE_CONFIG_SET_PATH), post(handle_set_config))
                .route(exchange_map.resolve(ROUTE_CONFIG_RELOAD_PATH), get(handle_reload_config))
                .route(exchange_map.resolve(ROUTE_TOKENS_GET_PATH), post(handle_get_tokens))
                .route(exchange_map.resolve(ROUTE_TOKENS_SET_PATH), post(handle_set_tokens))
                .route(exchange_map.resolve(ROUTE_TOKENS_ADD_PATH), post(handle_add_tokens))
                .route(exchange_map.resolve(ROUTE_TOKENS_DELETE_PATH), post(handle_delete_tokens))
                .route(exchange_map.resolve(ROUTE_TOKENS_MERGE_PATH), post(handle_merge_tokens))
                .route(
                    exchange_map.resolve(ROUTE_TOKENS_ALIAS_SET_PATH),
                    post(handle_set_tokens_alias),
                )
                .route(
                    exchange_map.resolve(ROUTE_TOKENS_PROFILE_UPDATE_PATH),
                    post(handle_update_tokens_profile),
                )
                .route(
                    exchange_map.resolve(ROUTE_TOKENS_CONFIG_VERSION_UPDATE_PATH),
                    post(handle_update_tokens_config_version),
                )
                .route(exchange_map.resolve(ROUTE_TOKENS_REFRESH_PATH), post(handle_refresh_tokens))
                .route(
                    exchange_map.resolve(ROUTE_TOKENS_STATUS_SET_PATH),
                    post(handle_set_tokens_status),
                )
                .route(
                    exchange_map.resolve(ROUTE_TOKENS_PROXY_SET_PATH),
                    post(handle_set_tokens_proxy),
                )
                .route(
                    exchange_map.resolve(ROUTE_TOKENS_TIMEZONE_SET_PATH),
                    post(handle_set_tokens_timezone),
                )
                .route(exchange_map.resolve(ROUTE_PROXIES_GET_PATH), post(handle_get_proxies))
                .route(exchange_map.resolve(ROUTE_PROXIES_SET_PATH), post(handle_set_proxies))
                .route(exchange_map.resolve(ROUTE_PROXIES_ADD_PATH), post(handle_add_proxy))
                .route(exchange_map.resolve(ROUTE_PROXIES_DELETE_PATH), post(handle_delete_proxies))
                .route(
                    exchange_map.resolve(ROUTE_PROXIES_SET_GENERAL_PATH),
                    post(handle_set_general_proxy),
                )
                .route(exchange_map.resolve(ROUTE_NTP_SYNC_ONCE_PATH), get(handle_ntp_sync_once))
                .route_layer(middleware::from_fn(admin_auth_middleware)),
        )
        .merge(
            Router::new()
                .without_v07_checks()
                .route(exchange_map.resolve(ROUTE_CPP_CONFIG_PATH), post(handle_cpp_config))
                .route(exchange_map.resolve(ROUTE_CPP_MODELS_PATH), post(handle_cpp_models))
                .route(exchange_map.resolve(ROUTE_FILE_UPLOAD_PATH), post(handle_upload_file))
                .route(exchange_map.resolve(ROUTE_FILE_SYNC_PATH), post(handle_sync_file))
                .route(exchange_map.resolve(ROUTE_CPP_STREAM_PATH), post(handle_stream_cpp))
                .route_layer(middleware::from_fn_with_state(state.clone(), cpp_auth_middleware)),
        )
        .route(exchange_map.resolve(ROUTE_RAW_MODELS_PATH), get(handle_raw_models))
        .route(exchange_map.resolve(ROUTE_MODELS_PATH), get(handle_models))
        .route(
            exchange_map.resolve(ROUTE_MESSAGES_PATH),
            post(handle_messages)
                .route_layer(middleware::from_fn_with_state(state.clone(), v1_auth_middleware)),
        )
        .route(
            exchange_map.resolve(ROUTE_CHAT_COMPLETIONS_PATH),
            post(handle_chat_completions)
                .route_layer(middleware::from_fn_with_state(state.clone(), v1_auth_middleware)),
        )
        .route(
            exchange_map.resolve(ROUTE_CLI_CHAT_COMPLETIONS_PATH),
            post(handle_cli_chat_completions)
                .route_layer(middleware::from_fn(admin_auth_middleware)),
        )
        .route(
            exchange_map.resolve(ROUTE_MESSAGES_COUNT_TOKENS_PATH),
            post(handle_messages_count_tokens)
                .route_layer(middleware::from_fn_with_state(state.clone(), v1_auth2_middleware)),
        )
        // .route(exchange_map.resolve(ROUTE_LOGS_PATH), get(handle_logs))
        .route(exchange_map.resolve(ROUTE_LOGS_GET_PATH), post(handle_get_logs))
        .route(exchange_map.resolve(ROUTE_LOGS_TOKENS_GET_PATH), post(handle_get_logs_tokens))
        .route(exchange_map.resolve(ROUTE_ENV_EXAMPLE_PATH), get(handle_env_example))
        .route(exchange_map.resolve(ROUTE_CONFIG_EXAMPLE_PATH), get(handle_config_example))
        // .route(exchange_map.resolve(ROUTE_CONFIG_PATH), get(handle_config_page))
        // .route(exchange_map.resolve(ROUTE_CONFIG_PATH), post(handle_config_update))
        // .route(exchange_map.resolve(ROUTE_STATIC_PATH), get(handle_static))
        // .route(exchange_map.resolve(ROUTE_ABOUT_PATH), get(handle_about))
        .route(exchange_map.resolve(ROUTE_LICENSE_PATH), get(handle_license))
        .route(exchange_map.resolve(ROUTE_README_PATH), get(handle_readme))
        // .route(exchange_map.resolve(ROUTE_API_PATH), get(handle_api_page))
        .route(exchange_map.resolve(ROUTE_GEN_UUID_PATH), get(handle_gen_uuid))
        .route(exchange_map.resolve(ROUTE_GEN_HASH_PATH), get(handle_gen_hash))
        .route(exchange_map.resolve(ROUTE_GEN_CHECKSUM_PATH), get(handle_gen_checksum))
        .route(
            exchange_map.resolve(ROUTE_GET_CHECKSUM_HEADER_PATH),
            get(handle_get_checksum_header),
        )
        // .route(exchange_map.resolve(ROUTE_BASIC_CALIBRATION_PATH), post(handle_basic_calibration))
        // .route(exchange_map.resolve(ROUTE_USER_INFO_PATH), post(handle_user_info))
        // .route(exchange_map.resolve(ROUTE_BUILD_KEY_PATH), get(handle_build_key_page))
        .route(exchange_map.resolve(ROUTE_BUILD_KEY_PATH), post(handle_build_key))
        .route(exchange_map.resolve(ROUTE_CONFIG_VERSION_GET_PATH), post(handle_get_config_version))
        // .route(exchange_map.resolve(ROUTE_TOKEN_UPGRADE_PATH), post(handle_token_upgrade))
        .route(exchange_map.resolve(ROUTE_TOKEN_PROFILE_GET_PATH), post(handle_get_token_profile));

    crate::core::route::init_endpoints(exchange_map.finish());

    for (path, func) in routes {
        backend = backend.route(path, get(func))
    }

    backend
        .layer(RequestBodyLimitLayer::new(parse_from_env("REQUEST_BODY_LIMIT", 2_000_000usize)))
        .layer(CorsLayer::permissive())
        .with_state(state)
}
