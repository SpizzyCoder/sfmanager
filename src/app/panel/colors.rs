use std::path::Path;
use tui::style::Color;

const DIR_COLOR: Color = Color::Blue;
const FILE_COLOR: Color = Color::White;

const IMAGE_COLOR: Color = Color::Magenta;
const AUDIO_COLOR: Color = Color::Cyan;
const ARCHIVE_COLOR: Color = Color::Red;
const VIDEO_COLOR: Color = Color::Magenta;

const IMAGE_EXTENSIONS: &'static [&'static str] = &[
    "jpg", "jpeg", "jpe", "png", "bmp", "svg", "eps", "gif", "ico", "webp",
];

const AUDIO_EXTENSIONS: &'static [&'static str] = &[
    "mp3", "oga", "opus", "m4a", "flac", "wav", "wma", "aac", "alac",
];

const ARCHIVE_EXTENSIONS: &'static [&'static str] = &[
    "iso", "tar", "bz2", "gz", "lz", "lz4", "lzma", "lzo", "rz", "xz", "z", "zst", "7z", "s7z",
    "rar", "tgz", "tbz2", "tlz", "txz", "zip", "zipx", "jar",
];

const VIDEO_EXTENSIONS: &'static [&'static str] = &[
    "webm", "mkv", "flv", "vob", "ogv", "ogg", "gifv", "avi", "mov", "qt", "wmv", "mp4", "m4v",
    "mp2", "mpv",
];

pub fn get_color(path: &Path) -> Color {
    if path.is_dir() {
        return DIR_COLOR;
    }

    let path_extension: String = {
        if let Some(os_str) = path.extension() {
            if let Some(str_extension) = os_str.to_str() {
                str_extension.to_lowercase()
            } else {
                return FILE_COLOR;
            }
        } else {
            return FILE_COLOR;
        }
    };

    for extension in IMAGE_EXTENSIONS {
        if path_extension == *extension {
            return IMAGE_COLOR;
        }
    }

    for extension in AUDIO_EXTENSIONS {
        if path_extension == *extension {
            return AUDIO_COLOR;
        }
    }

    for extension in ARCHIVE_EXTENSIONS {
        if path_extension == *extension {
            return ARCHIVE_COLOR;
        }
    }

    for extension in VIDEO_EXTENSIONS {
        if path_extension == *extension {
            return VIDEO_COLOR;
        }
    }

    return FILE_COLOR;
}