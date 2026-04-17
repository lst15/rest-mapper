use anyhow::Result;
use std::io;

pub async fn wait_for_enter() -> Result<()> {
    tokio::task::spawn_blocking(|| {
        let mut line = String::new();
        io::stdin().read_line(&mut line)?;
        Ok::<(), io::Error>(())
    })
    .await??;

    Ok(())
}
