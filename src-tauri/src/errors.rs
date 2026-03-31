use thiserror::Error;

use ffmpeg_next;
use image;
use rusqlite;
use std::io;
use tantivy;
use ureq;
use walkdir;
use whisper_rs;

#[derive(Debug, Error)]
pub enum FolioError {
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
    #[error("Whisper error: {0}")]
    WhisperError(#[from] whisper_rs::WhisperError),
    #[error("Subtitle parse error: {0}")]
    SubtitleParseError(String),
    #[error("Walk dir error: {0}")]
    WalkDirError(#[from] walkdir::Error),
    #[error("Media error: {0}")]
    MediaError(String),
    #[error("FFmpeg error: {0}")]
    FFmpegError(#[from] ffmpeg_next::Error),
    #[error("Tantivy error: {0}")]
    TantivyError(#[from] tantivy::TantivyError),
    #[error("Search error: {0}")]
    SearchError(String),
    #[error("HTTP error: {0}")]
    HttpError(#[from] ureq::Error),
    #[error("Database error: {0}")]
    DatabaseError(#[from] rusqlite::Error),
    #[error("Image error: {0}")]
    ImageError(#[from] image::ImageError),
}

impl serde::Serialize for FolioError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
