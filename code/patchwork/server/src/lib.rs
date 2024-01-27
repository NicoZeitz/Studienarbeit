use axum::{routing::get, Router};
use std::{net::SocketAddr, time::Duration};
use tower_http::{timeout::TimeoutLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    api::api_router,
    web::{index_handler, web_handler},
};

mod api;
mod web;

pub fn start_server(port: Option<u16>, public: bool) -> tokio::io::Result<()> {
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build()?;

    rt.block_on(async {
        let port = port.unwrap_or(3000);
        let host = if public { [0, 0, 0, 0] } else { [127, 0, 0, 1] };
        let addr = SocketAddr::from((host, port));

        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "example_graceful_shutdown=debug,tower_http=debug,axum=debug".into()),
            )
            .with(tracing_subscriber::fmt::layer().without_time())
            .init();

        let app = Router::new()
            .route("/", get(index_handler))
            .route("/index.html", get(index_handler))
            .nest("/api", api_router())
            .fallback_service(get(web_handler))
            .layer((TraceLayer::new_for_http(), TimeoutLayer::new(Duration::from_secs(10))));

        let listener = tokio::net::TcpListener::bind(addr.to_string()).await.unwrap();

        println!("Starting patchwork server on {}", addr);
        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal())
            .await
            .unwrap();
    });
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    println!("Received CTRL-C command. Exiting application...");
    std::process::exit(0);
}
