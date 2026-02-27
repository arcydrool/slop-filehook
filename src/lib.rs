use anyhow::Result;
use notify::{Watcher, RecursiveMode, watcher};
use sha2::{Sha256, Digest};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

/// Calculate SHA256 hash of a file
pub fn calculate_sha256(path: &std::path::Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 8192];
    
    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }
    
    Ok(hex::encode(hasher.finalize()))
}

/// Start monitoring the Downloads folder for new files
pub fn monitor_downloads() -> Result<()> {
    // Get the Downloads folder path
    let downloads_path = get_downloads_path()?;
    
    println!("Monitoring Downloads folder: {}", downloads_path.display());
    
    // Create a channel to receive file creation events
    let (tx, rx) = mpsc::channel();
    
    // Create a watcher for file system events
    let mut watcher = watcher(
        move |res: notify::Result<notify::Event>| {
            match res {
                Ok(event) => {
                    // Only care about Create events
                    if matches!(event.kind, notify::EventKind::Create(_)) {
                        for path in event.paths {
                            let _ = tx.send(path);
                        }
                    }
                }
                Err(e) => eprintln!("Watcher error: {}", e),
            }
        },
        notify::Config::default()
            .with_poll_interval(Duration::from_secs(1)),
    )?;
    
    // Watch the downloads directory recursively
    watcher.watch(&downloads_path, RecursiveMode::Recursive)?;
    
    // Process file creation events
    while let Ok(path) = rx.recv() {
        // Skip directories, only process files
        if path.is_file() {
            match calculate_sha256(&path) {
                Ok(sha256) => {
                    println!("File created: {}", path.display());
                    println!("  SHA256: {}", sha256);
                }
                Err(e) => {
                    eprintln!("Error calculating hash for {}: {}", path.display(), e);
                }
            }
        }
    }
    
    Ok(())
}

/// Get the Downloads folder path for the current user
fn get_downloads_path() -> Result<PathBuf> {
    let downloads = dirs::download_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine Downloads folder"))?;
    Ok(downloads)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_calculate_sha256() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, b"hello world")?;
        
        let sha = calculate_sha256(&test_file)?;
        // Known SHA256 of "hello world"
        assert_eq!(sha, "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9");
        
        Ok(())
    }
}
