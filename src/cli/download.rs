//! Download utilities for browsers and drivers

use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

pub struct Downloader {
    client: reqwest::Client,
}

impl Downloader {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub async fn download_file(&self, url: &str, dest: &Path) -> Result<()> {
        println!("Downloading from: {}", url);
        
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }

        let response = self.client.get(url).send().await?;
        
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Download failed with status: {}", response.status()));
        }

        let total_size = response.content_length().unwrap_or(0);
        let pb = ProgressBar::new(total_size);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .unwrap()
                .progress_chars("#>-"),
        );

        let mut file = File::create(dest)?;
        let mut downloaded: u64 = 0;
        let mut stream = response.bytes_stream();
        
        use futures::StreamExt;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk)?;
            downloaded += chunk.len() as u64;
            pb.set_position(downloaded);
        }

        pb.finish_with_message("Download complete");
        Ok(())
    }

    pub fn extract_zip(&self, archive: &Path, dest: &Path) -> Result<()> {
        println!("Extracting to: {:?}", dest);

        let file = File::open(archive)?;
        let mut archive = zip::ZipArchive::new(file)?;

        fs::create_dir_all(dest)?;

        let total = archive.len();
        let pb = ProgressBar::new(total as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} Extracting [{bar:40.cyan/blue}] {pos}/{len}")
                .unwrap()
                .progress_chars("#>-"),
        );

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            
            let outpath = match file.enclosed_name() {
                Some(path) => dest.join(path),
                None => continue,
            };

            if file.name().ends_with('/') {
                fs::create_dir_all(&outpath)?;
            } else {
                if let Some(p) = outpath.parent() {
                    fs::create_dir_all(p)?;
                }
                let mut outfile = File::create(&outpath)?;
                std::io::copy(&mut file, &mut outfile)?;
            }

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Some(mode) = file.unix_mode() {
                    fs::set_permissions(&outpath, fs::Permissions::from_mode(mode))?;
                }
            }

            pb.inc(1);
        }

        pb.finish_with_message("Extraction complete");
        Ok(())
    }

    pub async fn get_latest_chrome_version(&self) -> Result<String> {
        let url = "https://googlechromelabs.github.io/chrome-for-testing/last-known-good-versions.json";
        
        let response = self.client.get(url).send().await?;
        let json: serde_json::Value = response.json().await?;

        let version = json["channels"]["Stable"]["version"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Version not found"))?
            .to_string();

        Ok(version)
    }

    pub async fn install_chrome(&self, version: &str, platform_url: &str, install_dir: &Path) -> Result<PathBuf> {
        println!("Installing Chrome version {}", version);

        // Create a temp directory for download
        let temp_dir = install_dir.parent().unwrap_or(install_dir);
        let download_path = temp_dir.join(format!("chrome-{}.zip", version));
        self.download_file(platform_url, &download_path).await?;

        // Extract directly to install_dir
        self.extract_zip(&download_path, install_dir)?;

        fs::remove_file(&download_path)?;

        println!("Chrome installed successfully");
        Ok(install_dir.to_path_buf())
    }

    pub async fn install_chromedriver(&self, version: &str, platform_url: &str, install_dir: &Path) -> Result<PathBuf> {
        println!("Installing ChromeDriver version {}", version);

        // Create a temp directory for download
        let temp_dir = install_dir.parent().unwrap_or(install_dir);
        let download_path = temp_dir.join(format!("chromedriver-{}.zip", version));
        self.download_file(platform_url, &download_path).await?;

        // Extract directly to install_dir
        self.extract_zip(&download_path, install_dir)?;

        fs::remove_file(&download_path)?;

        println!("ChromeDriver installed successfully");
        Ok(install_dir.to_path_buf())
    }
}

impl Default for Downloader {
    fn default() -> Self {
        Self::new()
    }
}
