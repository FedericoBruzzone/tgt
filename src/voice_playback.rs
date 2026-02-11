//! Voice message and audio file playback using rodio.
//! Runs in a dedicated thread; position updates are sent to the main loop via actions.

use crate::action::Action;
use std::io::BufReader;
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug, Clone, Default)]
pub struct VoicePlaybackState {
    pub message_id: Option<i64>,
    pub position_secs: u64,
    pub duration_secs: u64,
    pub is_playing: bool,
}

#[derive(Debug)]
pub enum VoicePlaybackCommand {
    Play {
        path: String,
        duration_secs: u64,
        message_id: i64,
    },
    Stop,
}

/// Spawns the playback thread and returns the command sender.
/// Returns None if the audio output stream could not be opened (e.g. no ALSA on Linux ARM).
/// Stream and sink are created inside the thread (rodio OutputStream is not Send).
/// `wake_tx`: signalled when position (or other UI) is updated so the main loop can redraw immediately.
pub fn spawn_playback_thread(
    action_tx: UnboundedSender<Action>,
    wake_tx: tokio::sync::mpsc::UnboundedSender<()>,
) -> Option<std::sync::mpsc::Sender<VoicePlaybackCommand>> {
    let (cmd_tx, cmd_rx) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        let stream = match rodio::OutputStreamBuilder::open_default_stream() {
            Ok(s) => s,
            Err(e) => {
                tracing::error!("Failed to open default audio stream: {:?}", e);
                let _ = action_tx.send(Action::StatusMessage("Voice: no audio device".to_string()));
                return;
            }
        };
        run_playback_loop(cmd_rx, stream, action_tx, wake_tx);
    });

    Some(cmd_tx)
}

fn run_playback_loop(
    cmd_rx: Receiver<VoicePlaybackCommand>,
    stream: rodio::OutputStream, // must stay alive for playback
    action_tx: UnboundedSender<Action>,
    wake_tx: tokio::sync::mpsc::UnboundedSender<()>,
) {
    // Update often enough that the UI can show each second even when the main loop is occasionally slow.
    const POSITION_UPDATE_INTERVAL_MS: u64 = 250;
    let timeout = Duration::from_millis(POSITION_UPDATE_INTERVAL_MS);

    // Create a new Sink per play/stop so playback actually starts (reusing a stopped sink would not play).
    let mut sink = rodio::Sink::connect_new(stream.mixer());

    let mut current_message_id: Option<i64> = None;
    let mut current_duration_secs: u64 = 0;
    let mut playback_start = std::time::Instant::now(); // rodio 0.14 Sink has no get_pos()

    loop {
        match cmd_rx.recv_timeout(timeout) {
            Ok(VoicePlaybackCommand::Play {
                path,
                duration_secs,
                message_id,
            }) => {
                current_message_id = Some(message_id);
                current_duration_secs = duration_secs;
                // New Sink so playback actually starts.
                sink = rodio::Sink::connect_new(stream.mixer());

                if path.is_empty() {
                    let _ =
                        action_tx.send(Action::StatusMessage("Voice: no file path".to_string()));
                    let _ = action_tx.send(Action::VoicePlaybackEnded(message_id));
                    current_message_id = None;
                } else if let Ok(file) = std::fs::File::open(&path) {
                    let reader = BufReader::new(file);
                    // Try OGG Opus first (Telegram voice notes); then rodio Decoder (MP3, etc.)
                    let appended = match crate::ogg_opus::OpusSourceOgg::new(reader) {
                        Ok(opus_source) => {
                            playback_start = std::time::Instant::now();
                            sink.append(opus_source);
                            sink.play();
                            tracing::info!(
                                "Voice: playing OGG Opus, sink.empty()={}",
                                sink.empty()
                            );
                            let _ = action_tx.send(Action::VoicePlaybackStarted(message_id));
                            let _ = action_tx.send(Action::VoicePlaybackPosition(
                                message_id,
                                0,
                                duration_secs,
                            ));
                            true
                        }
                        Err(e) => {
                            tracing::debug!("Not OGG Opus ({:?}), trying rodio Decoder", e);
                            let file = match std::fs::File::open(&path) {
                                Ok(f) => f,
                                Err(_) => {
                                    let _ = action_tx.send(Action::StatusMessage(
                                        "Voice: re-open failed".to_string(),
                                    ));
                                    let _ = action_tx.send(Action::VoicePlaybackEnded(message_id));
                                    current_message_id = None;
                                    continue;
                                }
                            };
                            let reader = BufReader::new(file);
                            match rodio::Decoder::try_from(reader) {
                                Ok(source) => {
                                    playback_start = std::time::Instant::now();
                                    sink.append(source);
                                    sink.play();
                                    tracing::info!(
                                        "Voice: playing (rodio decoder), sink.empty()={}",
                                        sink.empty()
                                    );
                                    let _ =
                                        action_tx.send(Action::VoicePlaybackStarted(message_id));
                                    let _ = action_tx.send(Action::VoicePlaybackPosition(
                                        message_id,
                                        0,
                                        duration_secs,
                                    ));
                                    true
                                }
                                Err(dec_err) => {
                                    tracing::error!("Voice decode failed: {} {:?}", path, dec_err);
                                    let _ = action_tx.send(Action::StatusMessage(
                                        "Voice: decode failed (not OGG Opus?)".to_string(),
                                    ));
                                    false
                                }
                            }
                        }
                    };
                    if !appended {
                        let _ = action_tx.send(Action::VoicePlaybackEnded(message_id));
                        current_message_id = None;
                    }
                } else {
                    tracing::error!("Failed to open audio file: {}", path);
                    let _ = action_tx
                        .send(Action::StatusMessage("Voice: cannot open file".to_string()));
                    let _ = action_tx.send(Action::VoicePlaybackEnded(message_id));
                    current_message_id = None;
                }
            }
            Ok(VoicePlaybackCommand::Stop) => {
                // Replace with fresh sink so next Play gets a clean sink; dropping stops current sound.
                sink = rodio::Sink::connect_new(stream.mixer());
                current_message_id = None;
            }
            Err(RecvTimeoutError::Timeout) => {
                if let Some(msg_id) = current_message_id {
                    let empty = sink.empty();
                    if empty {
                        tracing::debug!("Voice: timeout, sink empty, sending Ended");
                        let _ = action_tx.send(Action::VoicePlaybackEnded(msg_id));
                        current_message_id = None;
                    } else {
                        let pos_secs = playback_start.elapsed().as_secs();
                        tracing::debug!("Voice: position {}s/{}s", pos_secs, current_duration_secs);
                        let _ = action_tx.send(Action::VoicePlaybackPosition(
                            msg_id,
                            pos_secs,
                            current_duration_secs,
                        ));
                        let _ = wake_tx.send(());
                    }
                }
            }
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }
}
