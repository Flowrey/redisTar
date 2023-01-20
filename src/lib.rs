use async_trait::async_trait;
use redis::AsyncCommands;
use tar::Header;
use tokio::{fs::File, io::{AsyncWriteExt, AsyncSeekExt, SeekFrom}};

pub struct RedisTar<'a> {
    pub con: &'a mut redis::aio::Connection,
    pub fd: &'a mut tokio::fs::File,
}

impl<'a> RedisTar<'a> {
    pub fn new(con: &'a mut redis::aio::Connection, fd: &'a mut tokio::fs::File) -> Self {
        Self { con, fd }
    }

    pub async fn append<B>(&mut self, path: &str, block: &mut B)
    where
        B: RedisAppend,
    {
        // Create header part
        let mut header = Header::new_gnu();
        header.set_path(path).unwrap();
        header.set_mode(0o0644);
        header.set_size(block.get_size());
        header.set_cksum();

        // Calculate block_size
        let remaining = 512 - (header.size().unwrap() % 512);
        let block_size = 512 + header.size().unwrap() + remaining;

        // Increment global block size in our Redis
        let offset: u64 = self.con.incr("covers.padding", block_size).await.unwrap();

        // Seek to the the begining of our offset
        self.fd.seek(SeekFrom::Start(offset - block_size)).await.unwrap();

        // Write header part to file (512 bytes)
        self.fd.write_all(header.as_bytes()).await.unwrap();

        // Write data part to disk
        block.write_content(&mut self.fd).await;

        // Pad with zeros if necessary.
        let buf = [0; 512];
        if remaining < 512 {
            self.fd.write_all(&buf[..remaining as usize]).await.unwrap();
        }

        // Verify if there is no corruption
        let current_offset = self.fd.seek(SeekFrom::Current(0)).await.unwrap();
        if current_offset != offset {
            panic!("invalid offset")
        }
    }
}

#[async_trait]
pub trait RedisAppend {
    fn get_size(&self) -> u64;
    async fn write_content(&mut self, fd: &mut File);
}

#[async_trait]
impl RedisAppend for reqwest::Response {
    fn get_size(&self) -> u64 {
        self.content_length().unwrap()
    }

    async fn write_content(&mut self, fd: &mut File) {
        while let Some(chunk) = self.chunk().await.unwrap() {
            fd.write_all(&chunk).await.unwrap();
        }
    }
}

#[async_trait]
impl RedisAppend for &[u8] {
    fn get_size(&self) -> u64 {
        self.len().try_into().unwrap()
    }

    async fn write_content(&mut self, fd: &mut File) {
        fd.write_all(&self).await.unwrap();
    }
}