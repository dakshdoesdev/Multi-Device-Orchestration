use anyhow::Result;
use std::process::Stdio;
use std::path::PathBuf;
use tokio::process::Command;
use tracing::info;

pub struct FastCapture {
    output_name: String,
    temp_dir: PathBuf,
    last_frame: Option<Vec<u8>>,
}

impl FastCapture {
    pub fn new() -> Result<Self> {
        let output_name = std::env::var("TABLET_OUTPUT").unwrap_or_else(|_| "TABLET-1".to_string());
        let temp_dir = std::env::temp_dir().join("tablet-display");
        std::fs::create_dir_all(&temp_dir)?;
        
        Ok(Self { 
            output_name, 
            temp_dir,
            last_frame: None,
        })
    }

    pub async fn start(&mut self) -> Result<()> {
        info!("⚡ Performance mode: LOW LATENCY");
        info!("   Resolution: 640x360 (very low)");
        info!("   Target: 30 FPS (reduced CPU load)");
        Ok(())
    }

    pub async fn capture_frame(&mut self) -> Result<Vec<u8>> {
        let output_path = self.temp_dir.join("frame.png");
        
        // Quick capture with grim
        let grim_result = tokio::time::timeout(
            std::time::Duration::from_millis(300),
            Command::new("grim")
                .args(&["-o", &self.output_name, "-t", "png", output_path.to_str().unwrap()])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
        ).await;

        match grim_result {
            Ok(Ok(status)) if status.success() => {
                if let Ok(data) = tokio::fs::read(&output_path).await {
                    if let Ok(jpeg) = self.fast_resize(&data).await {
                        self.last_frame = Some(jpeg.clone());
                        return Ok(jpeg);
                    }
                }
            }
            _ => {}
        }
        
        // Return last good frame or test pattern
        if let Some(ref last) = self.last_frame {
            Ok(last.clone())
        } else {
            Ok(self.generate_test_pattern().await)
        }
    }

    async fn fast_resize(&self, png_data: &[u8]) -> Result<Vec<u8>> {
        let png_data = png_data.to_vec();
        
        tokio::task::spawn_blocking(move || {
            let img = image::load_from_memory_with_format(&png_data, image::ImageFormat::Png)?;
            // VERY small resolution for speed
            let resized = img.resize(640, 360, image::imageops::FilterType::Nearest);
            
            let mut jpeg_data = Vec::new();
            let mut cursor = std::io::Cursor::new(&mut jpeg_data);
            // Low quality = fast
            let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut cursor, 40);
            encoder.encode_image(&resized)?;
            
            Ok::<_, anyhow::Error>(jpeg_data)
        }).await?
    }

    async fn generate_test_pattern(&self) -> Vec<u8> {
        tokio::task::spawn_blocking(|| {
            let mut img = image::RgbImage::new(640, 360);
            for y in 0..360 {
                for x in 0..640 {
                    img.put_pixel(x, y, image::Rgb([50, 50, 50]));
                }
            }
            let mut jpeg_data = Vec::new();
            let mut cursor = std::io::Cursor::new(&mut jpeg_data);
            let _ = img.write_to(&mut cursor, image::ImageFormat::Jpeg);
            jpeg_data
        }).await.unwrap_or_else(|_| vec![0u8; 100])
    }
}
