use std::{
    fs::File,
    io::{BufReader, Read, Seek, SeekFrom},
    str::FromStr,
    sync::{
        atomic::{AtomicBool, AtomicU32, Ordering},
        mpsc, Arc,
    },
    thread,
    time::Duration,
};

use bytes::Bytes;
use futures_util::StreamExt;
use log::info;
use reqwest::Error;

pub struct HttpStreamReader {
    output_reader: BufReader<File>,
    bytes_written: Arc<AtomicU32>,
    done: Arc<AtomicBool>,
    bytes_read: u32,
}

impl HttpStreamReader {
    pub fn new(url: String) -> Self {
        let bytes_written = Arc::new(AtomicU32::new(0));
        let done = Arc::new(AtomicBool::new(false));
        let bytes_written_ = bytes_written.clone();
        let done_ = done.clone();
        let (tx, rx) = mpsc::channel();

        tokio::spawn(async move {
            println!("starting download...");
            let client = reqwest::Client::new();
            let res = client.head(&url).send().await.unwrap();
            let length = res
                .headers()
                .get(reqwest::header::CONTENT_LENGTH)
                .ok_or("response doesn't include the content length")
                .unwrap();
            let length = u64::from_str(length.to_str().unwrap())
                .map_err(|_| "invalid Content-Length header")
                .unwrap();
            info!("File length={}", length);
            let mut output_file = File::create("download.bin").unwrap();
            let mut stream = client.get(&url).send().await.unwrap().bytes_stream();

            let mut append_file = |item: Result<Bytes, Error>| {
                let item = item.unwrap();
                let len = item.len();
                std::io::copy(&mut item.take(len as u64), &mut output_file).unwrap();
                // bytes_written_.store(
                //     bytes_written_.load(Ordering::SeqCst) + len as u32,
                //     Ordering::SeqCst,
                // );
            };

            if let Some(item) = stream.next().await {
                append_file(item);
                tx.send(()).unwrap();
            }

            while let Some(item) = stream.next().await {
                append_file(item);
            }

            done_.store(true, Ordering::SeqCst);
            println!("Finished with success!");
        });
        rx.recv().unwrap();
        //thread::sleep(Duration::from_secs(2));

        let output_file = File::open("download.bin").unwrap();
        let output_reader = BufReader::new(output_file);
        HttpStreamReader {
            output_reader,
            bytes_written,
            bytes_read: 0,
            done,
        }
    }
}

impl Read for HttpStreamReader {
    fn read(&mut self, mut buf: &mut [u8]) -> Result<usize, std::io::Error> {
        // while !self.done.load(Ordering::SeqCst)
        //     && self.bytes_written.load(Ordering::SeqCst) < self.bytes_read + buf.len() as u32
        // {
        //     thread::sleep(Duration::from_millis(1));
        // }
        let res = self.output_reader.read(&mut buf);
        self.bytes_read += buf.len() as u32;
        res
    }
}

impl Seek for HttpStreamReader {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, std::io::Error> {
        self.output_reader.seek(pos)
    }
}
