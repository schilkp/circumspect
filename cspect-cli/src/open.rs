use std::{
    path::Path,
    sync::{Arc, Condvar, Mutex},
    thread,
    time::{Duration, SystemTime},
};

use axum::{
    body::Body,
    http::{header, Response, StatusCode},
    routing::{get, post},
    Router,
};
use log::{debug, info};
use tokio::fs;
use tokio::sync::mpsc;
use tokio_util::io::ReaderStream;

const ORIGIN: &str = "https://ui.perfetto.dev";

async fn server(trace_file: &Path, notif_trace_served: Arc<Condvar>) {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let etag: String = ((now as u64) ^ 0xd3f4_0305_c9f8_e911_u64).to_string();

    let trace_path = trace_file.to_path_buf();
    let app = Router::new()
        .route(
            "/trace.proto",
            get(move || {
                let trace_path = trace_path.clone();
                let etag = etag.clone();
                let notif_trace_served = notif_trace_served.clone();
                async move {
                    match fs::File::open(&trace_path).await {
                        Ok(file) => {
                            let stream = ReaderStream::new(file);
                            let body = Body::from_stream(stream);
                            let resp = Response::builder()
                                .status(200)
                                .header(header::CONTENT_TYPE, "application/octet-stream")
                                .header(header::ETAG, etag)
                                .header("Access-Control-Allow-Origin", ORIGIN.to_string())
                                .body(body)
                                .unwrap();
                            notif_trace_served.notify_all();
                            info!("SERVER: Serving trace for /trace.proto GET request.");
                            resp.into_parts()
                        }
                        Err(e) => {
                            log::error!("Failed to open trace file: {}", e);
                            Response::builder()
                                .status(StatusCode::INTERNAL_SERVER_ERROR)
                                .body(Body::from("Failed to open trace file"))
                                .unwrap()
                                .into_parts()
                        }
                    }
                }
            }),
        )
        .route(
            "/status",
            post(|| async move {
                debug!("SERVER: Serving OK for status GET request.");
                StatusCode::OK
            }),
        );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:9001")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn start_trace_server(trace_file: &Path, temporary: bool) -> anyhow::Result<()> {
    if temporary {
        info!("Starting temporary trace-provider server..");
    } else {
        info!("Starting trace-provider server..");
    }

    let trace_file = trace_file.to_path_buf();
    let notif_trace_served = Arc::new(Condvar::new());
    let wait_trace_served = notif_trace_served.clone();

    let (send_stop, mut stop) = mpsc::channel::<()>(1);

    let server_thread = thread::spawn(move || {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                tokio::select! {
                    _ = server(&trace_file, notif_trace_served) => {
                    }
                    _ = stop.recv() => {
                    }
                }
            })
    });

    if temporary {
        let m: Mutex<()> = Mutex::new(());
        let l = m.lock().unwrap();
        let _lock = wait_trace_served.wait(l).unwrap();
        std::thread::sleep(Duration::from_millis(250));
        info!("Stopping server..");
        send_stop.blocking_send(()).unwrap();
    }

    let _ = server_thread.join();
    info!("Server stopped.");

    Ok(())
}

pub fn serve_trace(trace_file: &Path) -> anyhow::Result<()> {
    if !trace_file.exists() {
        return Err(anyhow::anyhow!(
            "Trace file does not exist: {}",
            trace_file.display()
        ));
    }
    let link = format!("{ORIGIN}/#!/?url=http://127.0.0.1:9001/trace.proto");
    info!("Serving trace.\n\n  Link: {link}\n");
    start_trace_server(trace_file, false)?;
    Ok(())
}

#[cfg(feature = "open")]
pub fn open_trace(trace_file: &Path) -> anyhow::Result<()> {
    if !trace_file.exists() {
        return Err(anyhow::anyhow!(
            "Trace file does not exist: {}",
            trace_file.display()
        ));
    }
    let link = format!("{ORIGIN}/#!/?url=http://127.0.0.1:9001/trace.proto");
    info!("Opening trace in perfetto..");
    webbrowser::open(&link)?;
    start_trace_server(trace_file, true)?;
    Ok(())
}
