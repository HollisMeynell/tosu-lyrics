use crate::error::Result;
use std::path::Path;

pub(crate) fn generate_random_string() -> String {
    use rand::distr::{Alphanumeric, SampleString};
    Alphanumeric.sample_string(&mut rand::rng(), 16)
}

#[cfg(target_os = "windows")]
const FFP_BLOB: &[u8] = include_bytes!("../lib/ffprobe.exe");

#[cfg(target_os = "linux")]
const FFP_BLOB: &[u8] = include_bytes!("../lib/ffprobe");

/// 返回毫秒数
pub(crate) async fn read_audio_length<P: AsRef<Path>>(file_path: P) -> Result<i32> {
    use tokio::fs::File;
    use tokio::io::AsyncWriteExt;
    use tokio::process::Command;

    let file_path = file_path.as_ref();

    if !file_path.try_exists().unwrap_or(false) {
        return Err(format!("音频文件不存在: {}", file_path.display()).into());
    }

    let ffprobe_path = Path::new("ffprobe");
    if !ffprobe_path.exists() {
        let mut ffprobe = File::create(ffprobe_path).await?;
        ffprobe.write_all(FFP_BLOB).await?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
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
            &*file_path.to_string_lossy(),
        ])
        .output()
        .await?;

    if !output.status.success() {
        return Err(format!(
            "ffprobe 执行失败: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    match stdout.parse::<f64>() {
        Ok(duration) if duration.is_finite() => Ok((duration * 1000.0).round() as i32),
        Ok(_) => Err("解析到无效的音频时长".into()),
        Err(e) => Err(format!("无法解析音频时长: {}", e).into()),
    }
}

#[cfg(test)]
mod test {
    use crate::util::generate_random_string;
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

    #[tokio::test]
    async fn test_generate_random_string() {
        let random_string = generate_random_string();
        assert_eq!(random_string.len(), 16);
        assert!(random_string.chars().all(|c| c.is_ascii_alphanumeric()));
        println!("Random string: {}", random_string);
    }
}
