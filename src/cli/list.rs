//! List command implementation

use anyhow::Result;
use directories::ProjectDirs;
use std::path::PathBuf;

pub async fn run() -> Result<()> {
    println!("Installed Browsers");
    println!("==================\n");

    let install_dir = get_install_dir()?;
    
    if !install_dir.exists() {
        println!("No browsers installed yet.");
        return Ok(());
    }

    let chrome_dir = install_dir.join("chrome");
    if chrome_dir.exists() {
        println!("✓ Chrome");
        println!("  Location: {:?}", chrome_dir);
    } else {
        println!("✗ Chrome (not installed)");
    }

    let driver_dir = install_dir.join("chromedriver");
    if driver_dir.exists() {
        println!("\n✓ ChromeDriver");
        println!("  Location: {:?}", driver_dir);
    } else {
        println!("\n✗ ChromeDriver (not installed)");
    }

    Ok(())
}

fn get_install_dir() -> Result<PathBuf> {
    // Use Playwright's cache directory structure for compatibility
    // This allows reusing browsers downloaded by Playwright
    if let Some(proj_dirs) = ProjectDirs::from("ms-playwright", "", "") {
        Ok(proj_dirs.cache_dir().to_path_buf())
    } else {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))?;
        Ok(PathBuf::from(home).join(".cache").join("ms-playwright"))
    }
}
