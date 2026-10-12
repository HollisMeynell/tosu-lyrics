use crate::error::{Error, Result};
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;
use symphonia::core::codecs::audio::AudioDecoderOptions;
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::probe::Hint;
use symphonia::core::formats::{FormatOptions, FormatReader, SeekMode, SeekTo, Track, TrackType};
use symphonia::core::io::{MediaSource, MediaSourceStream};
use symphonia::core::units::{Duration, TimeBase, Timestamp};

struct Source {
    file: File,
    len: u64,
    expose_length: bool,
}

impl Read for Source {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.file.read(buf)
    }
}

impl Seek for Source {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let target = match pos {
            SeekFrom::Start(pos) => i128::from(pos),
            SeekFrom::End(offset) => i128::from(self.len) + i128::from(offset),
            SeekFrom::Current(offset) => {
                i128::from(self.file.stream_position()?) + i128::from(offset)
            }
        };
        if target < 0 || target > i128::from(self.len) {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "音频数据超出文件边界",
            ));
        }
        self.file.seek(SeekFrom::Start(target as u64))
    }
}

impl MediaSource for Source {
    fn is_seekable(&self) -> bool {
        true
    }

    fn byte_len(&self) -> Option<u64> {
        self.expose_length.then_some(self.len)
    }
}

fn open(path: &Path, expose_length: bool) -> Result<Box<dyn FormatReader>> {
    let file = File::open(path)?;
    let meta = file.metadata()?;
    if !meta.is_file() || meta.len() == 0 {
        return Err("音频文件为空或不是普通文件".into());
    }
    let source = Source {
        file,
        len: meta.len(),
        expose_length,
    };
    let stream = MediaSourceStream::new(Box::new(source), Default::default());
    symphonia::default::get_probe()
        .probe(
            &Hint::new(),
            stream,
            FormatOptions::default(),
            Default::default(),
        )
        .map_err(|err| Error::Runtime(format!("无法探测音频格式（文件损坏或格式不支持）: {err}")))
}

fn track(format: &dyn FormatReader) -> Result<Track> {
    format
        .default_track(TrackType::Audio)
        .cloned()
        .ok_or_else(|| Error::Runtime("文件没有可识别的音轨".into()))
}

fn millis(ticks: u64, base: TimeBase) -> Result<i32> {
    let numerator = u128::from(ticks) * u128::from(base.numer.get()) * 1000;
    let denominator = u128::from(base.denom.get());
    let ms = (numerator + denominator / 2) / denominator;
    if ticks == 0 || ms == 0 {
        return Err("无法确定有效音频时长".into());
    }
    i32::try_from(ms).map_err(|_| Error::Runtime("音频时长超过接口支持的范围".into()))
}

fn metadata_duration(track: &Track) -> Option<(u64, TimeBase)> {
    if let (Some(base), Some(duration)) = (track.time_base, track.duration)
        && !duration.is_zero()
    {
        return Some((duration.get(), base));
    }
    let frames = track.num_frames.filter(|frames| *frames > 0)?;
    let rate = track.codec_params.as_ref()?.audio()?.sample_rate?;
    Some((frames, TimeBase::try_from_recip(rate)?))
}

pub(super) fn read(path: &Path) -> Result<i32> {
    let mut format = open(path, false)?;
    let expose_length = matches!(format.format_info().short_name, "ogg" | "caf");
    if expose_length {
        format = open(path, true)?;
    }
    let selected = track(&*format)?;
    if let Some((ticks, base)) = metadata_duration(&selected) {
        if validate_tail(&mut *format, &selected)? {
            return millis(ticks, base);
        }
        format = open(path, expose_length)?;
    }

    let Some(base) = selected.time_base else {
        return decode_duration(path, expose_length, selected.id);
    };
    let mut ticks = 0u64;
    let mut previous_end: Option<Timestamp> = None;
    let mut packets = 0u64;
    while let Some(packet) = format
        .next_packet()
        .map_err(|err| Error::Runtime(format!("音频 packet 读取失败（文件损坏或不完整）: {err}")))?
    {
        if packet.track_id != selected.id {
            continue;
        }
        packets += 1;
        if packet.dur.is_zero() {
            if packet.block_dur().is_zero() {
                return decode_duration(path, expose_length, selected.id);
            }
            continue;
        }
        ticks = ticks
            .checked_add(packet.dur.get())
            .ok_or_else(|| Error::Runtime("音频 packet 时长溢出".into()))?;
        let start = packet.pts.checked_add(packet.trim_start);
        let end = start.and_then(|start| start.checked_add(packet.dur));
        if start.is_none()
            || end.is_none()
            || previous_end.is_some_and(|previous| Some(previous) != start)
        {
            return decode_duration(path, expose_length, selected.id);
        }
        previous_end = end;
    }
    if packets == 0 {
        return Err("音轨没有完整的音频 packet".into());
    }
    if ticks > 0 {
        return millis(ticks, base);
    }
    decode_duration(path, expose_length, selected.id)
}

fn validate_tail(format: &mut dyn FormatReader, selected: &Track) -> Result<bool> {
    if !matches!(format.format_info().short_name, "wave" | "aiff" | "isomp4") {
        return Ok(true);
    }
    let Some(duration) = selected.duration.filter(|duration| !duration.is_zero()) else {
        return Ok(false);
    };
    let target = selected
        .start_ts
        .checked_add(Duration::new(duration.get() - 1))
        .ok_or("音轨结束时间戳溢出")?;
    match format.seek(
        SeekMode::Accurate,
        SeekTo::Timestamp {
            ts: target,
            track_id: selected.id,
        },
    ) {
        Ok(_) => {}
        Err(SymphoniaError::SeekError(_) | SymphoniaError::Unsupported(_)) => return Ok(false),
        Err(err) => return Err(format!("音频尾部定位失败（文件损坏或不完整）: {err}").into()),
    }
    while let Some(packet) = format
        .next_packet()
        .map_err(|err| Error::Runtime(format!("音频尾部读取失败（文件损坏或不完整）: {err}")))?
    {
        if packet.track_id != selected.id {
            continue;
        }
        let end = packet
            .pts
            .checked_add(packet.trim_start)
            .and_then(|start| start.checked_add(packet.dur))
            .ok_or("音频尾部时间戳溢出")?;
        if packet.pts <= target && end > target {
            return Ok(true);
        }
        if packet.pts > target {
            break;
        }
    }
    Err("音频实际数据未达到声明的时长，文件可能被截断".into())
}

fn decode_duration(path: &Path, expose_length: bool, track_id: u32) -> Result<i32> {
    let mut format = open(path, expose_length)?;
    let selected = format
        .tracks()
        .iter()
        .find(|track| track.id == track_id)
        .ok_or("重新读取时音轨发生变化")?;
    let params = selected
        .codec_params
        .as_ref()
        .and_then(|params| params.audio())
        .ok_or("音轨没有可用的音频编码参数")?;
    let options = AudioDecoderOptions::default().verify(true);
    let mut decoder = symphonia::default::get_codecs()
        .make_audio_decoder(params, &options)
        .map_err(|err| Error::Runtime(format!("音轨解码器不支持或初始化失败: {err}")))?;
    let mut frames = 0u64;
    let mut rate = 0u32;
    let mut nanos = 0u128;
    while let Some(packet) = format
        .next_packet()
        .map_err(|err| Error::Runtime(format!("音频 packet 读取失败（文件损坏或不完整）: {err}")))?
    {
        if packet.track_id != track_id {
            continue;
        }
        let audio = decoder
            .decode(&packet)
            .map_err(|err| Error::Runtime(format!("音频解码失败（文件损坏或不支持）: {err}")))?;
        let current_rate = audio.spec().rate();
        if current_rate == 0 {
            return Err("解码器返回了无效采样率".into());
        }
        if rate != 0 && rate != current_rate {
            nanos = nanos
                .checked_add(u128::from(frames) * 1_000_000_000 / u128::from(rate))
                .ok_or("解码音频时长溢出")?;
            frames = 0;
        }
        rate = current_rate;
        frames = frames
            .checked_add(audio.frames() as u64)
            .ok_or("解码采样数溢出")?;
    }
    if decoder.finalize().verify_ok == Some(false) {
        return Err("音频完整性校验失败".into());
    }
    if rate == 0 {
        return Err("无法解码出有效音频采样".into());
    }
    nanos = nanos
        .checked_add(u128::from(frames) * 1_000_000_000 / u128::from(rate))
        .ok_or("解码音频时长溢出")?;
    let ms = (nanos + 500_000) / 1_000_000;
    if ms == 0 {
        return Err("无法确定有效音频时长".into());
    }
    i32::try_from(ms).map_err(|_| Error::Runtime("音频时长超过接口支持的范围".into()))
}
