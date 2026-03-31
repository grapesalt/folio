use std::path::Path;

use ffmpeg_next::media::Type::{Audio, Subtitle, Video};

use crate::{errors::FolioError, subtitles, FolioResult};

pub struct RawFrame {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

impl RawFrame {
    pub fn decode_frame(
        frame: &ffmpeg_next::frame::Video,
        shrink_factor: u32,
    ) -> FolioResult<Self> {
        use ffmpeg_next::format::Pixel::RGBA;
        use ffmpeg_next::software::scaling::{self, flag::Flags};

        let width = (frame.width() / shrink_factor).max(1);
        let height = (frame.height() / shrink_factor).max(1);

        let mut scaler = scaling::Context::get(
            frame.format(),
            frame.width(),
            frame.height(),
            RGBA,
            width,
            height,
            Flags::FAST_BILINEAR,
        )?;

        let mut out = ffmpeg_next::frame::Video::empty();
        scaler.run(frame, &mut out)?;

        Ok(Self {
            data: out.data(0).to_vec(),
            width,
            height,
        })
    }

    pub fn to_base64_png(&self) -> FolioResult<String> {
        use base64::{engine::general_purpose::STANDARD, Engine};

        let img = image::RgbaImage::from_raw(
            self.width,
            self.height,
            self.data.clone(),
        )
        .ok_or_else(|| {
            FolioError::MediaError(
                "Failed to create image from raw frame".into(),
            )
        })?;

        let mut buf = std::io::Cursor::new(Vec::new());
        img.write_to(&mut buf, image::ImageFormat::Png)?;

        Ok(format!(
            "data:image/png;base64,{}",
            STANDARD.encode(buf.into_inner())
        ))
    }
}

pub fn extract_audio(path: &Path) -> FolioResult<Box<[f32]>> {
    ffmpeg_next::init()?;

    let mut ictx = ffmpeg_next::format::input(path)?;

    // TODO: Handle multiple audio streams.
    let stream = ictx.streams().best(Audio).ok_or_else(|| {
        FolioError::MediaError("No audio stream found".into())
    })?;

    let stream_idx = stream.index();

    let ctx = ffmpeg_next::codec::context::Context::from_parameters(
        stream.parameters(),
    )?;
    let mut decoder = ctx.decoder().audio()?;

    let mut resampler =
        ffmpeg_next::software::resampling::context::Context::get(
            decoder.format(),
            decoder.channel_layout(),
            decoder.rate(),
            ffmpeg_next::format::Sample::F32(
                ffmpeg_next::format::sample::Type::Packed,
            ),
            ffmpeg_next::util::channel_layout::ChannelLayout::MONO,
            16000,
        )?;

    let mut samples: Vec<f32> = Vec::new();

    for (stream, packet) in ictx.packets() {
        if stream.index() != stream_idx {
            continue;
        }
        decoder.send_packet(&packet)?;

        let mut decoded = ffmpeg_next::frame::Audio::empty();
        while decoder.receive_frame(&mut decoded).is_ok() {
            let mut resampled = ffmpeg_next::frame::Audio::empty();
            resampler.run(&decoded, &mut resampled)?;
            for chunk in resampled.data(0).chunks_exact(4) {
                samples.push(f32::from_le_bytes(chunk.try_into().unwrap()));
            }
        }
    }

    Ok(samples.into())
}

pub fn generate_thumbnail(
    video_path: &Path,
    ts: i64,
    shrink_factor: u32,
) -> FolioResult<String> {
    ffmpeg_next::init()?;

    let mut ictx = ffmpeg_next::format::input(video_path)?;

    let position = ts * 1_000_000;
    ictx.seek(position, ..position)?;

    let stream = ictx.streams().best(Video).ok_or_else(|| {
        FolioError::MediaError("No video stream found".into())
    })?;
    let stream_idx = stream.index();

    let ctx = ffmpeg_next::codec::context::Context::from_parameters(
        stream.parameters(),
    )?;

    let mut decoder = ctx.decoder().video()?;

    for (stream, packet) in ictx.packets() {
        if stream.index() != stream_idx {
            continue;
        }
        decoder.send_packet(&packet)?;

        let mut frame = ffmpeg_next::frame::Video::empty();

        if decoder.receive_frame(&mut frame).is_ok() {
            return RawFrame::decode_frame(&frame, shrink_factor)?
                .to_base64_png();
        }
    }

    Err(FolioError::MediaError(format!(
        "No video frame found at timestamp {ts}"
    )))
}

pub fn extract_subtitles(
    path: &Path,
) -> FolioResult<Box<[subtitles::Segment]>> {
    ffmpeg_next::init()?;
    let mut ictx = ffmpeg_next::format::input(&path)?;
    let mut entries = Vec::new();

    // TODO: Handle multiple subtitle streams.
    let stream =
        ictx.streams().best(Subtitle).ok_or(FolioError::MediaError(
            format!("No subtitle stream found for {}", path.display()),
        ))?;

    let stream_index = stream.index();
    let time_base = stream.time_base();

    let ctx = ffmpeg_next::codec::context::Context::from_parameters(
        stream.parameters(),
    )?;

    let mut decoder = ctx.decoder().subtitle()?;

    for (stream, packet) in ictx.packets() {
        if stream.index() != stream_index {
            continue;
        }

        let mut subtitle = ffmpeg_next::Subtitle::new();
        let _ = decoder.decode(&packet, &mut subtitle);

        let tb_num = time_base.numerator() as i64;
        let tb_den = time_base.denominator() as i64;

        let start = packet
            .pts()
            .unwrap_or(0)
            .saturating_mul(tb_num)
            .saturating_mul(1000)
            / tb_den;

        let end = start
            + packet
                .duration()
                .saturating_mul(tb_num)
                .saturating_mul(1000)
                / tb_den;

        for rect in subtitle.rects() {
            use ffmpeg_next::subtitle::Rect;

            let text = match rect {
                Rect::Text(t) => t.get().to_owned(),
                Rect::Ass(a) => {
                    let raw = a.get().to_owned();
                    match raw.splitn(10, ',').nth(9) {
                        Some(t) => t.trim().to_string(),
                        None => continue,
                    }
                }
                _ => continue,
            };

            if !text.is_empty() {
                entries.push(subtitles::Segment { start, end, text });
            }
        }
    }

    Ok(entries.into())
}
