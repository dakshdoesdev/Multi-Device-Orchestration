use axum::{
    extract::State,
    http::{header, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};
use tracing::{info, error};

mod capture;
use capture::FastCapture;

#[derive(Clone)]
struct AppState {
    latest_frame: Arc<Mutex<Option<Vec<u8>>>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("tablet_display=info")
        .init();

    info!("🚀 Starting Tablet Display Server (Optimized)");

    let state = AppState {
        latest_frame: Arc::new(Mutex::new(None)),
    };

    let frame_clone = state.latest_frame.clone();
    tokio::spawn(async move {
        if let Err(e) = run_capture(frame_clone).await {
            error!("Capture error: {}", e);
        }
    });

    let app = Router::new()
        .route("/", get(index_handler))
        .route("/frame.jpg", get(frame_handler))
        .route("/status", get(status_handler))
        .nest_service("/static", tower_http::services::ServeDir::new("static"))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    
    info!("");
    info!("🌐 Server running at http://0.0.0.0:8080");
    info!("");

    axum::serve(listener, app).await?;
    Ok(())
}

async fn run_capture(frame_store: Arc<Mutex<Option<Vec<u8>>>>) -> anyhow::Result<()> {
    let mut capture = FastCapture::new()?;
    capture.start().await?;
    
    let mut ticker = interval(Duration::from_millis(33)); // 30 FPS target
    
    loop {
        ticker.tick().await;
        
        if let Ok(frame_data) = capture.capture_frame().await {
            let mut store = frame_store.lock().await;
            *store = Some(frame_data);
        }
    }
}

async fn index_handler() -> impl IntoResponse {
    let html = include_str!("../static/index.html");
    ([(header::CONTENT_TYPE, "text/html")], html)
}

async fn frame_handler(State(state): State<AppState>) -> impl IntoResponse {
    let frame = state.latest_frame.lock().await;
    
    match frame.as_ref() {
        Some(data) => {
            let headers = [
                (header::CONTENT_TYPE, "image/jpeg"),
                (header::CACHE_CONTROL, "no-cache"),
            ];
            (StatusCode::OK, headers, data.clone()).into_response()
        }
        None => (StatusCode::SERVICE_UNAVAILABLE, "No frame").into_response(),
    }
}

async fn status_handler(State(state): State<AppState>) -> impl IntoResponse {
    let frame = state.latest_frame.lock().await;
    (StatusCode::OK, axum::Json(serde_json::json!({
        "running": true,
        "resolution": "640x360",
        "target_fps": 30,
        "has_frame": frame.is_some(),
    })))
}
