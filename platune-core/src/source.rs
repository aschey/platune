use std::{
    fmt::Display,
    io::{Read, Result, Seek, SeekFrom},
};

use symphonia::core::io::MediaSource;

pub struct ReadSeekSource<T: Read + Seek + Send> {
    inner: T,
    len: Option<u64>,
    name: String,
    pub extension: Option<String>,
}

impl<T: Read + Seek + Send> Display for ReadSeekSource<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)
    }
}

pub trait FileExt {
    fn get_file_ext(&self) -> Option<String>;
}

pub trait Source: MediaSource + FileExt + Display {
    fn as_media_source(self: Box<Self>) -> Box<dyn MediaSource>;
}

impl<T: Read + Seek + Send> ReadSeekSource<T> {
    /// Instantiates a new `ReadSeekSource<T>` by taking ownership and wrapping the provided
    /// `Read + Seek`er.
    pub fn new(inner: T, len: Option<u64>, name: String, extension: Option<String>) -> Self {
        ReadSeekSource {
            inner,
            len,
            name,
            extension,
        }
    }
}

impl<T: Read + Seek + Send> MediaSource for ReadSeekSource<T> {
    fn is_seekable(&self) -> bool {
        true
    }

    fn byte_len(&self) -> Option<u64> {
        self.len
    }
}

impl<T: Read + Seek + Send> Read for ReadSeekSource<T> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.inner.read(buf)
    }
}

impl<T: Read + Seek + Send> Seek for ReadSeekSource<T> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        self.inner.seek(pos)
    }
}

impl<T: Read + Seek + Send> FileExt for ReadSeekSource<T> {
    fn get_file_ext(&self) -> Option<String> {
        self.extension.clone()
    }
}

impl<T: Read + Seek + Send + 'static> Source for ReadSeekSource<T> {
    fn as_media_source(self: Box<Self>) -> Box<dyn MediaSource> {
        self
    }
}
