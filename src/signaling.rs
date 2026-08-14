use axum::extract::ws::{Message, WebSocket};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::mpsc;
use tracing::{info, warn, debug};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SignalMessage {
    #[serde(rename = "offer")]
    Offer { sdp: String },
    #[serde(rename = "answer")]
    Answer { sdp: String },
    #[serde(rename = "ice-candidate")]
    IceCandidate { candidate: String, sdp_mid: String, sdp_mline_index: u32 },
    #[serde(rename = "join")]
    Join { room: String },
    #[serde(rename = "peer-joined")]
    PeerJoined { peer_id: String },
    #[serde(rename = "peer-left")]
    PeerLeft { peer_id: String },
    #[serde(rename = "error")]
    Error { message: String },
}

pub struct Peer {
    id: String,
    tx: mpsc::UnboundedSender<Message>,
}

pub struct SignalingServer {
    peers: HashMap<String, Peer>,
}

impl SignalingServer {
    pub fn new() -> Self {
        Self {
            peers: HashMap::new(),
        }
    }

    pub async fn handle_connection(&mut self, mut socket: WebSocket) {
        let peer_id = Uuid::new_v4().to_string();
        info!("🔌 New peer connected: {}", peer_id);

        let (tx, mut rx) = mpsc::unbounded_channel::<Message>();
        
        self.peers.insert(peer_id.clone(), Peer {
            id: peer_id.clone(),
            tx: tx.clone(),
        });

        // Send welcome message
        let welcome = SignalMessage::PeerJoined { peer_id: peer_id.clone() };
        if let Ok(json) = serde_json::to_string(&welcome) {
            let _ = tx.send(Message::Text(json));
        }

        loop {
            tokio::select! {
                // Receive from WebSocket
                msg = socket.recv() => {
                    match msg {
                        Some(Ok(Message::Text(text))) => {
                            debug!("Received from {}: {}", peer_id, text);
                            
                            match serde_json::from_str::<SignalMessage>(&text) {
                                Ok(signal) => {
                                    self.handle_signal(&peer_id, signal).await;
                                }
                                Err(e) => {
                                    warn!("Failed to parse message: {}", e);
                                    let error = SignalMessage::Error { 
                                        message: format!("Invalid message: {}", e) 
                                    };
                                    if let Ok(json) = serde_json::to_string(&error) {
                                        let _ = tx.send(Message::Text(json));
                                    }
                                }
                            }
                        }
                        Some(Ok(Message::Close(_))) | None => {
                            info!("👋 Peer disconnected: {}", peer_id);
                            break;
                        }
                        Some(Err(e)) => {
                            warn!("WebSocket error for {}: {}", peer_id, e);
                            break;
                        }
                        _ => {}
                    }
                }
                // Send to WebSocket
                Some(msg) = rx.recv() => {
                    if socket.send(msg).await.is_err() {
                        break;
                    }
                }
            }
        }

        self.peers.remove(&peer_id);
        info!("🧹 Cleaned up peer: {}", peer_id);
    }

    async fn handle_signal(&self, from_peer: &str, signal: SignalMessage) {
        match signal {
            SignalMessage::Offer { ref sdp } => {
                info!("📡 Received offer from {}", from_peer);
                // In a multi-peer scenario, broadcast to others
                // For now, just log it
            }
            SignalMessage::Answer { ref sdp } => {
                info!("📡 Received answer from {}", from_peer);
            }
            SignalMessage::IceCandidate { ref candidate, .. } => {
                debug!("🧊 ICE candidate from {}: {}", from_peer, candidate);
            }
            SignalMessage::Join { room } => {
                info!("👥 Peer {} joined room: {}", from_peer, room);
            }
            _ => {}
        }
    }

    pub fn broadcast(&self, exclude: &str, message: SignalMessage) {
        if let Ok(json) = serde_json::to_string(&message) {
            for (id, peer) in &self.peers {
                if id != exclude {
                    let _ = peer.tx.send(Message::Text(json.clone()));
                }
            }
        }
    }
}
