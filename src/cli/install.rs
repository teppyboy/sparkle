//! Install command implementation

use super::{Downloader, Platform};
use anyhow::Result;
use directories::ProjectDirs;
use std::path::PathBuf;

pub async fn run(browser: &str, with_deps: bool, force: bool) -> Result<()> {
    println!("Sparkle Browser Installer");
    println!("=========================\n");

    let platform = Platform::detect()?;
    println!("Detected platform: {}", platform);

    let install_dir = get_install_dir()?;
    println!("Install directory: {:?}\n", install_dir);

    let downloader = Downloader::new();

    let version = downloader.get_latest_chrome_version().await?;
    println!("Latest stable version: {}\n", version);

    match browser.to_lowercase().as_str() {
        "chromium" | "chrome" => {
            install_chrome(&downloader, &platform, &version, &install_dir, force).await?;
            if with_deps {
                install_chromedriver(&downloader, &platform, &version, &install_dir, force).await?;
            }
        }
        "all" => {
            install_chrome(&downloader, &platform, &version, &install_dir, force).await?;
            install_chromedriver(&downloader, &platform, &version, &install_dir, force).await?;
        }
        _ => {
            return Err(anyhow::anyhow!("Unknown browser: {}", browser));
        }
    }

    println!("\nInstallation complete!");
    println!("\nTo use the installed browser:");
    println!("  Set CHROME_PATH={:?}", install_dir.join("chrome"));
    println!("  Set CHROMEDRIVER_PATH={:?}", install_dir.join("chromedriver"));

    Ok(())
}

async fn install_chrome(
    downloader: &Downloader,
    platform: &Platform,
    version: &str,
    install_dir: &PathBuf,
    force: bool,
) -> Result<()> {
    let chrome_dir = install_dir.join("chrome");
    
    if chrome_dir.exists() && !force {
        println!("Chrome is already installed. Use --force to reinstall.");
        return Ok(());
    }

    if chrome_dir.exists() {
        std::fs::remove_dir_all(&chrome_dir)?;
    }

    let url = platform.chrome_download_url(version);
    downloader.install_chrome(version, &url, install_dir).await?;

    Ok(())
}

async fn install_chromedriver(
    downloader: &Downloader,
    platform: &Platform,
    version: &str,
    install_dir: &PathBuf,
    force: bool,
) -> Result<()> {
    let driver_dir = install_dir.join("chromedriver");
    
    if driver_dir.exists() && !force {
        println!("ChromeDriver is already installed. Use --force to reinstall.");
        return Ok(());
    }

    if driver_dir.exists() {
        std::fs::remove_dir_all(&driver_dir)?;
    }

    let url = platform.chromedriver_download_url(version);
    downloader.install_chromedriver(version, &url, install_dir).await?;

    Ok(())
}

fn get_install_dir() -> Result<PathBuf> {
    if let Some(proj_dirs) = ProjectDirs::from("com", "sparkle", "browsers") {
        Ok(proj_dirs.data_dir().to_path_buf())
    } else {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))?;
        Ok(PathBuf::from(home).join(".sparkle").join("browsers"))
    }
}
