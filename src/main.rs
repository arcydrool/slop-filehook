use filehook::monitor_downloads;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    monitor_downloads()?;
    Ok(())
}
