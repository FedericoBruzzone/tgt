//! OGG Opus decoding for Telegram voice notes.
//! Uses `ogg` (container) and `opus` (libopus via audiopus_sys); implements rodio's `Source`.

use opus::{Channels, Decoder};
use std::io::{Read, Seek};
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

        const SAMPLE_RATE: u32 = 48_000;
        let decoder = Decoder::new(
            SAMPLE_RATE,
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

    /// Frame size in ms from Opus TOC (first byte of packet). See RFC 6716.
    fn frame_size_ms(toc: u8) -> Option<f32> {
        let c = toc >> 3;
        let _s = (toc >> 2) & 1; // 0 = mono frame, 1 = stereo frame (for frame size calc)
        let ms = match c {
            0 | 4 | 8 | 12 | 14 | 18 | 22 | 26 | 30 => 10.0,
            1 | 5 | 9 | 13 | 15 | 19 | 23 | 27 | 31 => 20.0,
            2 | 6 | 10 => 40.0,
            3 | 7 | 11 => 60.0,
            16 | 20 | 24 | 28 => 2.5,
            17 | 21 | 25 | 29 => 5.0,
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
            .decode_float(&packet.data, &mut out, false)
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

    fn channels(&self) -> u16 {
        self.channel_count as u16
    }

    fn sample_rate(&self) -> u32 {
        48_000
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}
