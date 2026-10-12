use crate::error::Result;
use serde::Deserialize;
use std::any::type_name;
use std::path::Path;

pub(crate) fn generate_random_string() -> String {
    use rand::distr::{Alphanumeric, SampleString};
    Alphanumeric.sample_string(&mut rand::rng(), 16)
}

mod audio_duration;

pub(crate) async fn read_audio_length<P: AsRef<Path>>(file_path: P) -> Result<i32> {
    let path = file_path.as_ref().to_owned();
    tokio::task::spawn_blocking(move || audio_duration::read(&path))
        .await
        .map_err(|err| crate::error::Error::Runtime(format!("音频解析任务失败: {err}")))?
}

pub(crate) fn to_json<'a, T: Deserialize<'a>>(value: &'a str) -> Result<T> {
    let mut de = serde_json::Deserializer::from_str(value);
    serde_path_to_error::deserialize(&mut de).map_err(|e| {
        let type_name = type_name::<T>();
        let path = e.path().to_string();
        let err = e.into_inner();
        crate::error::Error::Json(err).with_context(format!("{type_name}:{path}"))
    })
}

#[cfg(test)]
mod test {
    use crate::util::generate_random_string;
    use crate::util::read_audio_length;

    #[tokio::test]
    async fn test_get_audio_length() {
        #[cfg(windows)]
        let test_path =
            "C:/Users/benwa/AppData/Local/osu!/Songs/[DT1] 687493 Toyama Nao - Ima Koko/audio.mp3";
        #[cfg(unix)]
        let test_path = "/home/spring/Documents/match/osu/2041495/audio.mp3";
        match read_audio_length(test_path).await {
            Ok(len) => println!("音频长度: {}ms", len),
            Err(e) => {
                println!("错误详情: {:#}", e);
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
