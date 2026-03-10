//! OGG Opus decoding for Telegram voice notes.
//! Uses `ogg` (container) and `audiopus` (codec); implements rodio's `Source`.

use audiopus::{coder::Decoder, Channels};
use std::io::{Read, Seek};
use std::num::{NonZeroU16, NonZeroU32};
use std::time::Duration;

/// Minimal OGG Opus source: reads Opus packets from an OGG container and decodes to f32 samples.
/// Implements `rodio::Source` for playback.
pub struct OpusSourceOgg<R>
where
    R: Read + Seek,
{
    channel_count: u8,
    packet: ogg::PacketReader<R>,
    decoder: Decoder,
    buffer: Vec<f32>,
    buffer_pos: usize,
}

#[derive(Debug)]
pub enum OggOpusError {
    NotOpusHead,
    Ogg(ogg::OggReadError),
}

impl std::fmt::Display for OggOpusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OggOpusError::NotOpusHead => write!(f, "not an OGG Opus stream (missing OpusHead)"),
            OggOpusError::Ogg(e) => write!(f, "ogg: {}", e),
        }
    }
}

impl std::error::Error for OggOpusError {}

impl From<ogg::OggReadError> for OggOpusError {
    fn from(e: ogg::OggReadError) -> Self {
        OggOpusError::Ogg(e)
    }
}

impl<R> OpusSourceOgg<R>
where
    R: Read + Seek,
{
    /// Build an OGG Opus source from a reader (e.g. `BufReader<File>`).
    /// Reads the OpusHead and OpusTags packets, then yields decoded f32 samples from the following packets.
    pub fn new(reader: R) -> Result<Self, OggOpusError> {
        let mut packet_reader = ogg::PacketReader::new(reader);
        let id_header = packet_reader.read_packet_expected()?.data;
        if id_header.len() < 19 || &id_header[0..8] != b"OpusHead" {
            return Err(OggOpusError::NotOpusHead);
        }
        let channel_count = id_header[9];
        let _comment_header = packet_reader.read_packet_expected()?.data;

        let decoder = Decoder::new(
            audiopus::SampleRate::Hz48000,
            if channel_count == 1 {
                Channels::Mono
            } else {
                Channels::Stereo
            },
        )
        .map_err(|_| OggOpusError::NotOpusHead)?;

        Ok(Self {
            channel_count,
            packet: packet_reader,
            decoder,
            buffer: vec![],
            buffer_pos: 0,
        })
    }

    fn next_packet(&mut self) -> Option<ogg::Packet> {
        while let Ok(packet) = self.packet.read_packet_expected() {
            if !packet.data.is_empty() {
                return Some(packet);
            }
        }
        None
    }

    /// Decodes the Opus TOC (Table of Contents) byte to get this packet's frame duration in milliseconds.
    ///
    /// Every Opus packet starts with a TOC byte (RFC 6716, section 3.2). The upper 5 bits form a
    /// configuration index `c = toc >> 3` that determines bandwidth, mode, and **frame size**.
    /// We need the frame size to allocate the right output buffer before calling the decoder:
    /// `num_samples = (sample_rate * frame_size_ms / 1000) * channels`. Opus uses six possible
    /// frame sizes (2.5, 5, 10, 20, 40, 60 ms) depending on `c`.
    ///
    /// **How we decode it:** The spec groups the 32 config indices into three bands. In each band,
    /// the frame size follows a fixed pattern, so we use a small lookup table and an index derived
    /// from `c` (see comments below). The `_s` bit (toc >> 2 & 1) is the stereo flag; we don't
    /// need it for duration. Returns `None` only for reserved/invalid `c` (e.g. c > 31).
    fn frame_size_ms(toc: u8) -> Option<f32> {
        let c = toc >> 3; // config index 0..31
        let _s = (toc >> 2) & 1; // stereo flag; unused for duration

        // RFC 6716 Table 7: frame size (ms) by config index band.
        // Band 1 (c 0–11): pattern 10, 20, 40, 60 repeated → index = c % 4
        const FRAME_MS_0_11: [f32; 4] = [10.0, 20.0, 40.0, 60.0];
        // Band 2 (c 12–15): pattern 10, 20, 10, 20 → index = c - 12
        const FRAME_MS_12_15: [f32; 4] = [10.0, 20.0, 10.0, 20.0];
        // Band 3 (c 16–31): reduced-delay pattern 2.5, 5, 10, 20 repeated → index = (c - 16) % 4
        const FRAME_MS_16_31: [f32; 4] = [2.5, 5.0, 10.0, 20.0];

        let ms = match c {
            0..=11 => FRAME_MS_0_11[c as usize % 4],
            12..=15 => FRAME_MS_12_15[(c - 12) as usize],
            16..=31 => FRAME_MS_16_31[(c - 16) as usize % 4],
            _ => return None,
        };
        Some(ms)
    }

    fn next_chunk(&mut self) -> Option<Vec<f32>> {
        let packet = self.next_packet()?;
        let toc = *packet.data.first()?;
        let frame_size_ms = Self::frame_size_ms(toc)?;
        let channels = self.channel_count as usize;
        let sample_rate = 48_000u32;
        let num_samples = (sample_rate as f32 / (1000.0 / frame_size_ms)) as usize * channels;
        let mut out = vec![0.0f32; num_samples];
        self.decoder
            .decode_float(Some(&packet.data), &mut out, false)
            .ok()?;
        Some(out)
    }
}

impl<R> Iterator for OpusSourceOgg<R>
where
    R: Read + Seek,
{
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        if self.buffer.is_empty() {
            self.buffer = self.next_chunk()?;
            self.buffer_pos = 0;
        }
        if self.buffer_pos < self.buffer.len() {
            let v = self.buffer[self.buffer_pos];
            self.buffer_pos += 1;
            return Some(v);
        }
        self.buffer.clear();
        self.next()
    }
}

#[cfg(feature = "voice-message")]
impl<R> rodio::Source for OpusSourceOgg<R>
where
    R: Read + Seek + Send + 'static,
{
    fn current_span_len(&self) -> Option<usize> {
        if self.buffer_pos < self.buffer.len() {
            Some(self.buffer.len() - self.buffer_pos)
        } else {
            None
        }
    }

    fn channels(&self) -> NonZeroU16 {
        NonZeroU16::new(self.channel_count as u16).unwrap_or(NonZeroU16::MIN)
    }

    fn sample_rate(&self) -> NonZeroU32 {
        NonZeroU32::new(48_000).expect("sample rate is non-zero")
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}
