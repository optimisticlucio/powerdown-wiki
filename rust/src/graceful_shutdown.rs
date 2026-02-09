use crate::scheduled_tasks::run_backup_processes;
use crate::ServerState;
use tokio::signal;

pub async fn handle_shutdown_signal(state: ServerState) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("[GRACEFUL SHUTDOWN] failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("[GRACEFUL SHUTDOWN] failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    println!("[GRACEFUL SHUTDOWN] Recieved shutdown command and existing connections handled! Initiating graceful shutdown protocol.");

    run_backup_processes(&state).await;

    println!("[GRACEFUL SHUTDOWN] Shutdown operation complete. Goodbye!");
}
