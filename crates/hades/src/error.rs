use std::io::Error;

use rtrb::chunks::ChunkError;
use thiserror::Error;
use wasapi::WasapiError;

#[derive(Error, Debug)]
pub enum HadesError {
    #[error("Generic I/O Error: {0}")]
    GenericIOError(#[from] Error),

    #[error("WASAPI Error: {0}")]
    WASAPIInternalError(#[from] WasapiError),

    #[error("Ring Buffer Chunk Error: {0}")]
    RingBufferChunkError(#[from] ChunkError),

    #[error("Audio device not found by ID: {0}")]
    DeviceNotFoundByID(String),

    #[error("Audio device not found by provided full/partial friendly name: {0}")]
    DeviceNotFoundByFriendlyName(String),

    #[error("Loopback Bridge Capture AudioClient was not initialized")]
    LoopbackBridgeCaptureAudioClientNotInitialized,

    #[error("Loopback Bridge Capture AudioCaptureClient was not initialized")]
    LoopbackBridgeCaptureAudioCaptureClientNotInitialized,

    #[error("Loopback Bridge Render AudioClient was not initialized")]
    LoopbackBridgeRenderAudioClientNotInitialized,

    #[error("Loopback Bridge Render AudioRenderClient was not initialized")]
    LoopbackBridgeRenderAudioRenderClientNotInitialized,

    #[error("Loopback Bridge Capture Ring Buffer was not initialized")]
    LoopbackBridgeCaptureRingBufferNotInitialized,

    #[error("Loopback Bridge Render Ring Buffer was not initialized")]
    LoopbackBridgeRenderRingBufferNotInitialized,
}
