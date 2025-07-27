// 直接一个 type alias

type BaseLyricSetter = super::super::LyricLinePayload;

impl BaseLyricSetter {
    pub fn new(first: &str, second: &str) -> BaseLyricSetter {
        Self {
            first: Some(first.to_string()),
            second: Some(second.to_string()),
        }
    }
}
