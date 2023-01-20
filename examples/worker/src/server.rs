use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .init();

    let redis_client = redis::Client::open("redis://127.0.0.1/")?;
    let mut con = redis_client.get_async_connection().await?;

    let cover_to_download = [
        "https://coverartarchive.org/release-group/ea8e2fec-ed89-3918-a9e4-bf4e8e5f7e28/front",
        "https://coverartarchive.org/release-group/3351433a-72a3-4c58-9fc5-9c628a35742e/front",
        "https://ia802905.us.archive.org/34/items/mbid-2a0eff02-5ba8-4932-850a-91bfc0bcab89/mbid-2a0eff02-5ba8-4932-850a-91bfc0bcab89-26119448481.jpg",
        "https://ia601702.us.archive.org/2/items/mbid-1637ac31-0dde-4552-8701-269d5d6730e1/mbid-1637ac31-0dde-4552-8701-269d5d6730e1-6991166746.jpg",
        "https://ia801706.us.archive.org/31/items/mbid-c02bf707-80fc-43c2-b34d-5fcf381b8668/mbid-c02bf707-80fc-43c2-b34d-5fcf381b8668-6991140908.jpg",
    ];

    let mut pipe = redis::pipe();
    for url in cover_to_download {
        let mut cmd = redis::cmd("LPUSH");
        pipe.add_command(cmd.arg("covers").arg(url).clone());
    }
    let _ : () = pipe.cmd("PING").query_async(&mut con).await.unwrap();
    info!("sending urls");
    Ok(())
}
