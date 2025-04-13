use crate::error::Result;

pub(crate) fn generate_random_string() -> String {
    use rand::Rng;
    use rand::distr::Alphanumeric;
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect()
}

#[cfg(target_os = "windows")]
const FFP_BLOB: &[u8] = include_bytes!("../lib/ffprobe.exe");

#[cfg(target_os = "linux")]
const FFP_BLOB: &[u8] = include_bytes!("../lib/ffprobe");

/// 返回毫秒数
pub(crate) async fn read_audio_length(file_path: &str) -> Result<i32> {
    use std::os::unix::fs::PermissionsExt;
    use std::path::Path;
    use tokio::fs::File;
    use tokio::io::AsyncWriteExt;
    use tokio::process::Command;
    let ffprobe_path = Path::new("ffprobe");
    if !ffprobe_path.exists() {
        let mut ffprobe = File::create(ffprobe_path).await?;
        ffprobe.write(FFP_BLOB).await?;
        #[cfg(unix)]
        {
            let mut perms = ffprobe.metadata().await?.permissions();
            perms.set_mode(0o755);
            ffprobe.set_permissions(perms).await?;
        }
    }
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
            file_path,
        ])
        .output()
        .await?;
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    match stdout.parse::<f64>() {
        Ok(duration) if duration.is_finite() => Ok((duration * 1000.0).round() as i32),
        _ => Err("length is NaN".into()),
    }
}

#[cfg(test)]
mod test {
    use crate::util::read_audio_length;

    #[tokio::test]
    async fn test_get_audio_length() {
        match read_audio_length("/home/spring/Documents/match/osu/2041495/audio.mp3").await {
            Ok(len) => {
                println!("audio len: {len}");
            }
            Err(err) => {
                println!("has error: {err}");
            }
        }
    }
}
