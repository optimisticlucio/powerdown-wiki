use powerdown_wiki::{
    handle_shutdown_signal, initiate_scheduled_tasks, run_migrations, ServerState,
};
use std::{env, net::SocketAddr};

#[tokio::main]
async fn main() {
    println!("[STARTUP] Power Down Wiki starting up...");

    let state = ServerState::initalize().await;
    println!("[STARTUP] ServerState initialized");

    if env::var("DISABLE_MIGRATIONS").is_ok() {
        println!("[STARTUP] Skipping SQL migrations!");
    } else {
        match run_migrations(&state).await {
            Ok(report) => {
                let applied_migrations = report.applied_migrations();

                if applied_migrations.is_empty() {
                    println!("[STARTUP] No SQL migrations applied!");
                } else {
                    let readable_migrations = applied_migrations
                        .iter()
                        .map(|migration| migration.name())
                        .collect::<Vec<&str>>()
                        .join(",");

                    println!("[STARTUP] SQL migrations applied: {readable_migrations}");
                }
            }
            Err(err) => {
                eprintln!("[STARTUP] Migrations failed to run! Terminating server. Err: {err:?}",);
                return;
            }
        }
    }

    let app = powerdown_wiki::router(state.clone());
    println!("[STARTUP] Created main router");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();

    initiate_scheduled_tasks(state.clone());
    println!("[STARTUP] Scheduled tasks initialized");

    println!("[STARTUP] Serving website!");
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(handle_shutdown_signal(state))
    .await
    .unwrap();
}
