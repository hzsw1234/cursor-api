use crate::common::model::HeaderValue;

/// 定义 HeaderName 常量
///
/// # Example
/// ```
/// def_header_name! {
///     (HEADER_NAME_X_REQUEST_ID, "x-request-id"),
///     (HEADER_NAME_X_TRACE_ID, "x-trace-id"),
/// }
/// ```
macro_rules! def_header_name {
    ($(($const_name:ident, $value:expr)),+ $(,)?) => {
        $(
            pub(crate) const $const_name: http::header::HeaderName =
                http::header::HeaderName::from_static($value);
        )+
    };
}

/// 定义 HeaderValue 常量
///
/// # Example
/// ```
/// def_header_value! {
///     (HEADER_VALUE_APPLICATION_JSON, "application/json"),
///     (HEADER_VALUE_TEXT_PLAIN, "text/plain"),
/// }
/// ```
macro_rules! def_header_value {
    ($(($const_name:ident, $value:expr)),+ $(,)?) => {
        $(
            pub const $const_name: http::header::HeaderValue = http::header::HeaderValue::from_static($value);
        )+
    };
}

def_header_value! {
    (NONE, ""),
    (ZERO, "0"),
    (ONE, "1"),
    (FALSE, "false"),
    (TRUE, "true"),
    (ENCODING, "gzip"),
    (ENCODINGS, "gzip, deflate, br, zstd"),
    (HEADER_VALUE_ACCEPT, "*/*"),
    (LANGUAGE, "en-US"),
    (EMPTY, "empty"),
    (CORS, "cors"),
    (NO_CACHE, "no-cache"),
    (NO_CACHE_REVALIDATE, "no-cache, must-revalidate"),
    (SAME_ORIGIN, "same-origin"),
    (KEEP_ALIVE, "keep-alive"),
    (CLOSE, "close"),
    (TRAILERS, "trailers"),
    (U_EQ_0, "u=0"),
    (U_EQ_1_I, "u=1, i"),
    (CONNECT_ES, "connect-es/2.1.1"),
    (NOT_A_BRAND, "\"Not)A;Brand\";v=\"8\", \"Chromium\";v=\"140\""),
    (MOBILE_NO, "?0"),
    (VSCODE_ORIGIN, "vscode-file://vscode-app"),
    (CROSS_SITE, "cross-site"),
    (EVENT_STREAM, "text/event-stream"),
    (CHUNKED, "chunked"),
    (JSON, "application/json"),
    (PROTO, "application/proto"),
    (CONNECT_PROTO, "application/connect+proto"),
    (CURSOR_ORIGIN, "https://cursor.com"),
    (CURSOR_REFERER_URL, "https://cursor.com/dashboard"),
    // (CURSOR_API2_HOST, super::CURSOR_API2_HOST),
    // (CURSOR_HOST, super::CURSOR_HOST),
    // (CURSOR_API4_HOST, super::CURSOR_API4_HOST),
    // (CURSOR_GCPP_ASIA_HOST, super::CURSOR_GCPP_ASIA_HOST),
    // (CURSOR_GCPP_EU_HOST, super::CURSOR_GCPP_EU_HOST),
    // (CURSOR_GCPP_US_HOST, super::CURSOR_GCPP_US_HOST),
}

def_header_name! {
    (PROXY_HOST, "x-co"),
    (CONFIG_HASH, "x-config-hash"),
    (API_KEY, "x-api-key"),
    (SESSION_ID, "x-session-id"),
    (GHOST_MODE, "x-ghost-mode"),
    (CONNECT_ACCEPT_ENCODING, "connect-accept-encoding"),
    (CONNECT_CONTENT_ENCODING, "connect-content-encoding"),
    (CONNECT_PROTOCOL_VERSION, "connect-protocol-version"),
    (AMZN_TRACE_ID, "x-amzn-trace-id"),
    (CLIENT_KEY, "x-client-key"),
    (CURSOR_CHECKSUM, "x-cursor-checksum"),
    (CURSOR_CLIENT_VERSION, "x-cursor-client-version"),
    (CURSOR_CONFIG_VERSION, "x-cursor-config-version"),
    (CURSOR_STREAMING, "x-cursor-streaming"),
    (CURSOR_TIMEZONE, "x-cursor-timezone"),
    (FS_CLIENT_KEY, "x-fs-client-key"),
    (REQUEST_ID, "x-request-id"),
    (NEW_ONBOARDING_COMPLETED, "x-new-onboarding-completed"),
    (SEC_CH_UA, "sec-ch-ua"),
    (SEC_CH_UA_MOBILE, "sec-ch-ua-mobile"),
    (SEC_CH_UA_PLATFORM, "sec-ch-ua-platform"),
    (SEC_FETCH_DEST, "sec-fetch-dest"),
    (SEC_FETCH_MODE, "sec-fetch-mode"),
    (SEC_FETCH_SITE, "sec-fetch-site"),
    (SEC_GPC, "sec-gpc"),
    (PRIORITY, "priority"),
    (STAINLESS_OS, "x-stainless-os"),
    (STAINLESS_ARCH, "x-stainless-arch"),
}

#[allow(unused_imports)]
pub use http::header::{CONNECTION, HOST, TRANSFER_ENCODING};

macro_rules! def_content_type {
    // 递归终点：所有项已处理完
    (@parse [$($with_header:tt)*] [$($without_header:tt)*]) => {
        def_content_type!(@generate_without_header $($without_header)*);
        def_content_type!(@generate_with_header $($with_header)*);
    };

    // 处理 => 语法（有逗号）
    (@parse [$($with_header:tt)*] [$($without_header:tt)*] $name:ident => $value:expr, $($rest:tt)*) => {
        def_content_type!(@parse [$($with_header)* $name => $value,] [$($without_header)*] $($rest)*);
    };

    // 处理 => 语法（无逗号）
    (@parse [$($with_header:tt)*] [$($without_header:tt)*] $name:ident => $value:expr) => {
        def_content_type!(@parse [$($with_header)* $name => $value,] [$($without_header)*]);
    };

    // 处理 = 语法（有逗号）
    (@parse [$($with_header:tt)*] [$($without_header:tt)*] $name:ident = $value:expr, $($rest:tt)*) => {
        def_content_type!(@parse [$($with_header)*] [$($without_header)* $name = $value,] $($rest)*);
    };

    // 处理 = 语法（无逗号）
    (@parse [$($with_header:tt)*] [$($without_header:tt)*] $name:ident = $value:expr) => {
        def_content_type!(@parse [$($with_header)*] [$($without_header)* $name = $value,]);
    };

    // 生成字符串常量
    (@generate_without_header $($name:ident = $value:expr,)*) => {
        $(paste::paste! {
            const [<$name:upper>]: &'static str = $value;
        })*
    };

    // 生成字符串常量和 HeaderValue 常量
    (@generate_with_header $($name:ident => $value:expr,)*) => {
        $(paste::paste! {
            const [<$name:upper>]: &'static str = $value;
            pub const [<HEADER_VALUE_ $name:upper>]: http::header::HeaderValue =
                http::header::HeaderValue::from_static([<$name:upper>]);
        })*
    };

    ($($item:tt)*) => {
        def_content_type!(@parse [] [] $($item)*);
    };
}

// Content type constants
def_content_type!(
    // 文本类型
    text_html_utf8 => "text/html;charset=utf-8",
    text_plain_utf8 => "text/plain;charset=utf-8",
    text_css_utf8 = "text/css;charset=utf-8",
    application_js_utf8 = "application/javascript;charset=utf-8",
    text_csv_utf8 = "text/csv;charset=utf-8",
    text_xml_utf8 = "text/xml;charset=utf-8",
    text_markdown_utf8 = "text/markdown;charset=utf-8",

    // 图像类型
    image_jpeg = "image/jpeg",
    image_png = "image/png",
    image_gif = "image/gif",
    image_webp = "image/webp",
    image_svg_xml = "image/svg+xml",
    image_bmp = "image/bmp",
    image_ico = "image/x-icon",
    image_tiff = "image/tiff",
    image_avif = "image/avif",

    // 音频类型
    audio_mpeg = "audio/mpeg",
    audio_mp4 = "audio/mp4",
    audio_wav = "audio/wav",
    audio_ogg = "audio/ogg",
    audio_webm = "audio/webm",
    audio_aac = "audio/aac",
    audio_flac = "audio/flac",
    audio_m4a = "audio/m4a",

    // 视频类型
    video_mp4 = "video/mp4",
    video_mpeg = "video/mpeg",
    video_webm = "video/webm",
    video_ogg = "video/ogg",
    video_avi = "video/x-msvideo",
    video_quicktime = "video/quicktime",
    video_x_flv = "video/x-flv",

    // 应用程序文档格式
    application_pdf = "application/pdf",
    application_msword = "application/msword",
    application_word_docx = "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
    application_excel_xls = "application/vnd.ms-excel",
    application_excel_xlsx = "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
    application_powerpoint_ppt = "application/vnd.ms-powerpoint",
    application_powerpoint_pptx = "application/vnd.openxmlformats-officedocument.presentationml.presentation",

    // 压缩文件
    application_zip = "application/zip",
    application_rar = "application/x-rar-compressed",
    application_7z = "application/x-7z-compressed",
    application_gzip = "application/gzip",
    application_tar = "application/x-tar",

    // 字体类型
    font_ttf = "font/ttf",
    font_otf = "font/otf",
    font_woff = "font/woff",
    font_woff2 = "font/woff2",

    // 通用二进制流
    application_octet_stream = "application/octet-stream"
);

// 文件扩展名到 HeaderValue 的映射
static EXTENSION_TO_MIME: phf::Map<&'static str, &'static str> = phf::phf_map! {
    // 文本类型
    "html" => TEXT_HTML_UTF8,
    "htm" => TEXT_HTML_UTF8,
    "txt" => TEXT_PLAIN_UTF8,
    "css" => TEXT_CSS_UTF8,
    "js" => APPLICATION_JS_UTF8,
    "mjs" => APPLICATION_JS_UTF8,
    "csv" => TEXT_CSV_UTF8,
    "xml" => TEXT_XML_UTF8,
    "md" => TEXT_MARKDOWN_UTF8,
    "markdown" => TEXT_MARKDOWN_UTF8,

    // 图像类型
    "jpg" => IMAGE_JPEG,
    "jpeg" => IMAGE_JPEG,
    "png" => IMAGE_PNG,
    "gif" => IMAGE_GIF,
    "webp" => IMAGE_WEBP,
    "svg" => IMAGE_SVG_XML,
    "bmp" => IMAGE_BMP,
    "ico" => IMAGE_ICO,
    "tiff" => IMAGE_TIFF,
    "tif" => IMAGE_TIFF,
    "avif" => IMAGE_AVIF,

    // 音频类型
    "mp3" => AUDIO_MPEG,
    "mp4a" => AUDIO_MP4,
    "wav" => AUDIO_WAV,
    "ogg" => AUDIO_OGG,
    "oga" => AUDIO_OGG,
    "weba" => AUDIO_WEBM,
    "aac" => AUDIO_AAC,
    "flac" => AUDIO_FLAC,
    "m4a" => AUDIO_M4A,

    // 视频类型
    "mp4" => VIDEO_MP4,
    "mpeg" => VIDEO_MPEG,
    "mpg" => VIDEO_MPEG,
    "webm" => VIDEO_WEBM,
    "ogv" => VIDEO_OGG,
    "avi" => VIDEO_AVI,
    "mov" => VIDEO_QUICKTIME,
    "qt" => VIDEO_QUICKTIME,
    "flv" => VIDEO_X_FLV,

    // 应用程序文档格式
    "pdf" => APPLICATION_PDF,
    "doc" => APPLICATION_MSWORD,
    "docx" => APPLICATION_WORD_DOCX,
    "xls" => APPLICATION_EXCEL_XLS,
    "xlsx" => APPLICATION_EXCEL_XLSX,
    "ppt" => APPLICATION_POWERPOINT_PPT,
    "pptx" => APPLICATION_POWERPOINT_PPTX,

    // 压缩文件
    "zip" => APPLICATION_ZIP,
    "rar" => APPLICATION_RAR,
    "7z" => APPLICATION_7Z,
    "gz" => APPLICATION_GZIP,
    "gzip" => APPLICATION_GZIP,
    "tar" => APPLICATION_TAR,

    // 字体类型
    "ttf" => FONT_TTF,
    "otf" => FONT_OTF,
    "woff" => FONT_WOFF,
    "woff2" => FONT_WOFF2,
};

/// 根据文件扩展名获取对应的 Content-Type HeaderValue
pub fn get_content_type_by_extension(extension: &str) -> http::header::HeaderValue {
    let extension = if extension.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()) {
        alloc::borrow::Cow::Borrowed(extension)
    } else {
        alloc::borrow::Cow::Owned(extension.to_ascii_lowercase())
    };
    unsafe {
        HeaderValue::from_static(
            EXTENSION_TO_MIME.get(&*extension).copied().unwrap_or(APPLICATION_OCTET_STREAM),
        )
        .into()
    }
}

#[derive(Clone, Copy, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentType {
    TextHtmlUtf8,
    TextPlainUtf8,
    TextCssUtf8,
    ApplicationJavascriptUtf8,
    TextCsvUtf8,
    TextXmlUtf8,
    TextMarkdownUtf8,
    ImageJpeg,
    ImagePng,
    ImageGif,
    ImageWebp,
    ImageSvgXml,
    ImageBmp,
    ImageIco,
    ImageTiff,
    ImageAvif,
    AudioMpeg,
    AudioMp4,
    AudioWav,
    AudioOgg,
    AudioWebm,
    AudioAac,
    AudioFlac,
    AudioM4a,
    VideoMp4,
    VideoMpeg,
    VideoWebm,
    VideoOgg,
    VideoAvi,
    VideoQuicktime,
    VideoXFlv,
    ApplicationPdf,
    ApplicationMsword,
    ApplicationWordDocx,
    ApplicationExcelXls,
    ApplicationExcelXlsx,
    ApplicationPowerpointPpt,
    ApplicationPowerpointPptx,
    ApplicationZip,
    ApplicationRar,
    Application7z,
    ApplicationGzip,
    ApplicationTar,
    FontTtf,
    FontOtf,
    FontWoff,
    FontWoff2,
    ApplicationOctetStream,
}

impl From<ContentType> for http::HeaderValue {
    #[inline]
    fn from(ty: ContentType) -> Self {
        let s = match ty {
            ContentType::TextHtmlUtf8 => TEXT_HTML_UTF8,
            ContentType::TextPlainUtf8 => TEXT_PLAIN_UTF8,
            ContentType::TextCssUtf8 => TEXT_CSS_UTF8,
            ContentType::ApplicationJavascriptUtf8 => APPLICATION_JS_UTF8,
            ContentType::TextCsvUtf8 => TEXT_CSV_UTF8,
            ContentType::TextXmlUtf8 => TEXT_XML_UTF8,
            ContentType::TextMarkdownUtf8 => TEXT_MARKDOWN_UTF8,
            ContentType::ImageJpeg => IMAGE_JPEG,
            ContentType::ImagePng => IMAGE_PNG,
            ContentType::ImageGif => IMAGE_GIF,
            ContentType::ImageWebp => IMAGE_WEBP,
            ContentType::ImageSvgXml => IMAGE_SVG_XML,
            ContentType::ImageBmp => IMAGE_BMP,
            ContentType::ImageIco => IMAGE_ICO,
            ContentType::ImageTiff => IMAGE_TIFF,
            ContentType::ImageAvif => IMAGE_AVIF,
            ContentType::AudioMpeg => AUDIO_MPEG,
            ContentType::AudioMp4 => AUDIO_MP4,
            ContentType::AudioWav => AUDIO_WAV,
            ContentType::AudioOgg => AUDIO_OGG,
            ContentType::AudioWebm => AUDIO_WEBM,
            ContentType::AudioAac => AUDIO_AAC,
            ContentType::AudioFlac => AUDIO_FLAC,
            ContentType::AudioM4a => AUDIO_M4A,
            ContentType::VideoMp4 => VIDEO_MP4,
            ContentType::VideoMpeg => VIDEO_MPEG,
            ContentType::VideoWebm => VIDEO_WEBM,
            ContentType::VideoOgg => VIDEO_OGG,
            ContentType::VideoAvi => VIDEO_AVI,
            ContentType::VideoQuicktime => VIDEO_QUICKTIME,
            ContentType::VideoXFlv => VIDEO_X_FLV,
            ContentType::ApplicationPdf => APPLICATION_PDF,
            ContentType::ApplicationMsword => APPLICATION_MSWORD,
            ContentType::ApplicationWordDocx => APPLICATION_WORD_DOCX,
            ContentType::ApplicationExcelXls => APPLICATION_EXCEL_XLS,
            ContentType::ApplicationExcelXlsx => APPLICATION_EXCEL_XLSX,
            ContentType::ApplicationPowerpointPpt => APPLICATION_POWERPOINT_PPT,
            ContentType::ApplicationPowerpointPptx => APPLICATION_POWERPOINT_PPTX,
            ContentType::ApplicationZip => APPLICATION_ZIP,
            ContentType::ApplicationRar => APPLICATION_RAR,
            ContentType::Application7z => APPLICATION_7Z,
            ContentType::ApplicationGzip => APPLICATION_GZIP,
            ContentType::ApplicationTar => APPLICATION_TAR,
            ContentType::FontTtf => FONT_TTF,
            ContentType::FontOtf => FONT_OTF,
            ContentType::FontWoff => FONT_WOFF,
            ContentType::FontWoff2 => FONT_WOFF2,
            ContentType::ApplicationOctetStream => APPLICATION_OCTET_STREAM,
        };
        unsafe { HeaderValue::from_static(s).into() }
    }
}
