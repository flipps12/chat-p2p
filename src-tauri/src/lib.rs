use tauri::{ Emitter, State };
use knot_sdk::{ KnotClient, KnotCommand };
use tokio::sync::Mutex;
use std::{ sync::Arc, time::{ SystemTime, UNIX_EPOCH } };

// Estructura para compartir el cliente con los comandos de Tauri
struct KnotState {
    client: Mutex<Option<KnotClient>>,
}

#[tauri::command]
async fn send_knot_command(
    state: State<'_, Arc<KnotState>>,
    command: String,
    args: Vec<String>
) -> Result<(), String> {
    let mut client_guard = state.client.lock().await;
    let client = client_guard.as_mut().ok_or("KnotClient no está conectado todavía")?;

    let cmd = match command.as_str() {
        "status" => KnotCommand::Status,
        "version" => KnotCommand::Version,
        "connect" => KnotCommand::Connect { multiaddr: args[0].clone() },
        "pees" => KnotCommand::GetPeers,
        "ping" => {
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
            let text = now.as_nanos().to_string();
            return client.send_bytes(&args[0], text.as_bytes(), 0).await.map_err(|e| e.to_string());
        }
        _ => {
            return Err("Comando no reconocido".into());
        }
    };

    client
        .send_json(cmd).await
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn send_message_command(
    state: State<'_, Arc<KnotState>>,
    message: String,
    peerid: String
) -> Result<(), String> {
    let mut client_guard = state.client.lock().await;
    let client = client_guard.as_mut().ok_or("KnotClient no está conectado todavía")?;

    println!("Message: '{message}' to peerid: '{peerid}'");

    let _ = client
        .send_bytes(&peerid, message.as_bytes(), 2000).await;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 1. Creamos el estado FUERA del builder para tener control total
    let knot_state = Arc::new(KnotState {
        client: Mutex::new(None),
    });

    tauri::Builder
        ::default()
        .plugin(tauri_plugin_opener::init())
        // 2. Pasamos el clon del Arc al manejador de estados de Tauri
        .manage(knot_state.clone())
        .setup(move |app| {
            let app_handle = app.handle().clone();

            // 3. Pasamos el clon del Arc directamente al hilo asíncrono
            // Ya no necesitamos llamar a app_handle.state() dentro del spawn
            let inner_state = knot_state.clone();

            tauri::async_runtime::spawn(async move {
                let knot_client = match KnotClient::new(7565).await {
                    Ok(k) => k,
                    Err(e) => {
                        eprintln!("Failed to connect KnotClient: {e}");
                        return;
                    }
                };

                // 4. Ahora es seguro llenar el cliente
                {
                    let mut client_guard = inner_state.client.lock().await;
                    *client_guard = Some(knot_client.clone());
                }

                // 5. Lanzamos los listeners
                start_knot_listeners(app_handle, knot_client).await;
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![send_knot_command, send_message_command])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn start_knot_listeners(app_handle: tauri::AppHandle, knot: KnotClient) {
    let mut msg_rx = knot.subscribe_messages();

    // El registro inicial
    let _ = knot.send_json(KnotCommand::Register { app_id: 2000, port: 7565 }).await;

    // Tu loop de mensajes
    let h = app_handle.clone();
    tokio::spawn(async move {
        while let Ok(msg) = msg_rx.recv().await {
            if let Some(response) = msg.response {
                if !response.is_null() {
                    // Emitimos al frontend
                    h.emit("knot-response", response).unwrap();
                }
            }
        }
    });

    // Tu loop de bytes (RTT)
    let mut byte_rx = knot.subscribe_bytes();
    let h2 = app_handle.clone();
    tokio::spawn(async move {
        while let Ok(msg) = byte_rx.recv().await {
            if let Ok(sent) = timing::parse_timestamp(&msg) {
                let rtt = timing::diff_ms(sent);
                h2.emit("knot-rtt", rtt).unwrap();
            }
        }
    });
}

pub mod timing {
    use std::time::{ SystemTime, UNIX_EPOCH, Duration };

    /// Convierte texto (u64) → Duration desde UNIX_EPOCH
    pub fn parse_timestamp(text: &str) -> Result<Duration, String> {
        let value: u64 = text
            .trim()
            .parse()
            .map_err(|_| "Invalid timestamp".to_string())?;

        Ok(Duration::from_nanos(value))
    }

    /// Devuelve "ahora" como Duration desde UNIX_EPOCH
    pub fn now() -> Duration {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap()
    }

    /// Diferencia en milisegundos
    pub fn diff_ms(sent: Duration) -> u128 {
        let now = now();
        now.saturating_sub(sent).as_millis()
    }

    /// Diferencia en microsegundos
    pub fn diff_us(sent: Duration) -> u128 {
        let now = now();
        now.saturating_sub(sent).as_micros()
    }

    /// Diferencia en nanosegundos
    pub fn diff_ns(sent: Duration) -> u128 {
        let now = now();
        now.saturating_sub(sent).as_nanos()
    }
}
