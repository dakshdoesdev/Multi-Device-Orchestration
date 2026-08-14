use anyhow::Result;
use std::sync::Arc;
use tracing::{info, debug, error};
use webrtc::{
    api::{
        interceptor_registry::register_default_interceptors,
        media_engine::{MediaEngine, MIME_TYPE_H264, MIME_TYPE_VP8},
        APIBuilder,
    },
    ice_transport::ice_server::RTCIceServer,
    interceptor::registry::Registry,
    peer_connection::{
        configuration::RTCConfiguration,
        peer_connection_state::RTCPeerConnectionState,
        RTCPeerConnection,
    },
    rtp_transceiver::rtp_codec::RTCRtpCodecCapability,
    track::track_local::track_local_static_sample::TrackLocalStaticSample,
};

pub struct WebRTCManager {
    api: Arc<webrtc::api::API>,
}

impl WebRTCManager {
    pub fn new() -> Result<Self> {
        let mut m = MediaEngine::default();
        
        // Register default codecs
        m.register_default_codecs()?;
        
        // Create interceptor registry
        let mut registry = Registry::new();
        registry = register_default_interceptors(registry, &mut m)?;
        
        let api = Arc::new(
            APIBuilder::new()
                .with_media_engine(m)
                .with_interceptor_registry(registry)
                .build(),
        );
        
        Ok(Self { api })
    }

    pub async fn create_peer_connection(&self) -> Result<Arc<RTCPeerConnection>> {
        let config = RTCConfiguration {
            ice_servers: vec![
                RTCIceServer {
                    urls: vec!["stun:stun.l.google.com:19302".to_string()],
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let pc = self.api.new_peer_connection(config).await?;
        
        // Set up connection state handler
        let pc_clone = Arc::clone(&pc);
        pc.on_peer_connection_state_change(Box::new(move |state| {
            let pc_clone = Arc::clone(&pc_clone);
            Box::pin(async move {
                info!("Connection state changed: {:?}", state);
                
                if state == RTCPeerConnectionState::Failed {
                    let _ = pc_clone.close().await;
                }
            })
        }));

        Ok(pc)
    }

    pub async fn add_video_track(
        &self,
        pc: &Arc<RTCPeerConnection>,
        codec: &str,
    ) -> Result<Arc<TrackLocalStaticSample>> {
        let codec_capability = match codec {
            "h264" => RTCRtpCodecCapability {
                mime_type: MIME_TYPE_H264.to_string(),
                clock_rate: 90000,
                channels: 0,
                sdp_fmtp_line: "level-asymmetry-allowed=1;packetization-mode=1;profile-level-id=42001f".to_string(),
                rtcp_feedback: vec![],
            },
            "vp8" => RTCRtpCodecCapability {
                mime_type: MIME_TYPE_VP8.to_string(),
                clock_rate: 90000,
                channels: 0,
                sdp_fmtp_line: "".to_string(),
                rtcp_feedback: vec![],
            },
            _ => anyhow::bail!("Unsupported codec: {}", codec),
        };

        let video_track = Arc::new(TrackLocalStaticSample::new(
            codec_capability,
            "video".to_string(),
            "screen-share".to_string(),
        ));

        let _ = pc
            .add_track(Arc::clone(&video_track) as Arc<dyn webrtc::track::track_local::TrackLocal + Send + Sync>)
            .await?;

        Ok(video_track)
    }
}

/// Video track that reads from screen capture and sends to WebRTC
pub struct VideoStreamer {
    track: Arc<TrackLocalStaticSample>,
}

impl VideoStreamer {
    pub fn new(track: Arc<TrackLocalStaticSample>) -> Self {
        Self { track }
    }

    pub async fn start(&self) -> Result<()> {
        // This would read from the capture channel and write samples
        // to the WebRTC track
        info!("🎬 Video streamer started");
        
        // Placeholder - actual implementation would:
        // 1. Read encoded H264 frames from the encoder
        // 2. Package as RTP samples
        // 3. Write to track
        
        Ok(())
    }
}

/// Check if WebRTC can be used (check for required dependencies)
pub fn check_webrtc_deps() -> bool {
    info!("Checking WebRTC dependencies...");
    
    // Check for OpenSSL (required by webrtc-rs)
    let openssl = std::process::Command::new("openssl")
        .arg("version")
        .output();
    
    match openssl {
        Ok(out) if out.status.success() => {
            let version = String::from_utf8_lossy(&out.stdout);
            info!("✅ OpenSSL: {}", version.trim());
            true
        }
        _ => {
            error!("❌ OpenSSL not found - required for WebRTC");
            false
        }
    }
}
