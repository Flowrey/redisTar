use redis::AsyncCommands;
use tokio::fs::OpenOptions;
use tracing::info;

use aiotar::RedisTar;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Open redis connection
    let redis_client = redis::Client::open("redis://127.0.0.1/")?;
    let mut con = redis_client.get_async_connection().await?;

    // Create or open archive
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open("covers.tar")
        .await?;

    // Create a RedisTar
    let mut redis_tar = RedisTar::new(&mut con, &mut file);

    loop {
        let (_, url): (String, String) = redis_tar.con.blpop("covers", 0).await.unwrap();
        let index: u64 = redis_tar.con.incr("covers.index", 1).await.unwrap();
        info!("downloading cover_000{}", index);

        let mut res = reqwest::get(&url).await.unwrap();
        redis_tar
            .append(&format!("cover_000{}", index), &mut res)
            .await;
    }
}
