use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::fs;
use std::io;
use anyhow::{Context, Result};

use crate::types::{PeerInfoToSave, PeerIdToSave, ChannelInfoToSave};

// todo, add personal config file for user preferences (Name, theme, etc)

// ============================================================================
// MANAGER DE ARCHIVOS
// ============================================================================

pub struct FileManager {
    base_dir: PathBuf,
}

impl FileManager {
    /// Crear nuevo FileManager con el directorio base apropiado según el OS
    pub fn new() -> Result<Self> {
        let base_dir = Self::get_app_data_dir()?;
        
        // Crear directorio si no existe
        fs::create_dir_all(&base_dir)
            .context("Failed to create app data directory")?;
        
        println!("📁 App data directory: {}", base_dir.display());
        
        Ok(Self { base_dir })
    }

    /// Obtener directorio de datos de la aplicación según el OS
    fn get_app_data_dir() -> Result<PathBuf> {
        #[cfg(target_os = "linux")]
        {
            // Linux: ~/.local/share/chat-p2p
            let home = std::env::var("HOME")
                .context("HOME environment variable not set")?;
            Ok(PathBuf::from(home).join(".local/share/chat-p2p"))
        }

        #[cfg(target_os = "windows")]
        {
            // Windows: C:\Users\<User>\AppData\Roaming\chat-p2p
            let appdata = std::env::var("APPDATA")
                .context("APPDATA environment variable not set")?;
            Ok(PathBuf::from(appdata).join("chat-p2p"))
        }

        #[cfg(target_os = "macos")]
        {
            // macOS: ~/Library/Application Support/chat-p2p
            let home = std::env::var("HOME")
                .context("HOME environment variable not set")?;
            Ok(PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("chat-p2p"))
        }

        #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
        {
            // Fallback para otros OS
            Ok(PathBuf::from("./chat-p2p-data"))
        }
    }

    /// Obtener ruta completa de un archivo
    fn get_file_path(&self, filename: &str) -> PathBuf {
        self.base_dir.join(filename)
    }

    // ========================================================================
    // PEERS
    // ========================================================================

    /// Cargar lista de peers conocidos
    pub fn load_peers(&self) -> Result<Vec<PeerInfoToSave>> {
        let path = self.get_file_path("peers.json");
        
        if !path.exists() {
            println!("📋 No peers file found, starting fresh");
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&path)
            .context("Failed to read peers file")?;
        
        let peers: Vec<PeerInfoToSave> = serde_json::from_str(&content)
            .context("Failed to parse peers file")?;
        
        println!("✅ Loaded {} peers", peers.len());
        Ok(peers)
    }

    /// Guardar lista de peers
    pub fn save_peers(&self, peers: &[PeerInfoToSave]) -> Result<()> {
        let path = self.get_file_path("peers.json");
        let content = serde_json::to_string_pretty(peers)
            .context("Failed to serialize peers")?;
        
        fs::write(&path, content)
            .context("Failed to write peers file")?;
        
        println!("💾 Saved {} peers", peers.len());
        Ok(())
    }

    /// Agregar o actualizar un peer
    pub fn add_or_update_peer(&self, peer: PeerInfoToSave) -> Result<()> {
        let mut peers = self.load_peers()?;
        
        // Buscar si ya existe
        if let Some(existing) = peers.iter_mut().find(|p| p.peer_id == peer.peer_id) {
            *existing = peer;
            println!("🔄 Updated peer: {}", existing.peer_id);
        } else {
            println!("➕ Added new peer: {}", peer.peer_id);
            peers.push(peer);
        }
        
        self.save_peers(&peers)
    }

    /// Eliminar un peer
    pub fn remove_peer(&self, peer_id: &str) -> Result<()> {
        let mut peers = self.load_peers()?;
        peers.retain(|p| p.peer_id != peer_id);
        
        println!("🗑️ Removed peer: {}", peer_id);
        self.save_peers(&peers)
    }

    /// Incrementar intentos fallidos de un peer
    pub fn increment_peer_failed_attempts(&self, peer_id: &str) -> Result<()> {
        let mut peers = self.load_peers()?;
        
        if let Some(peer) = peers.iter_mut().find(|p| p.peer_id == peer_id) {
            peer.failed_attempts += 1;
            println!("⚠️ Peer {} failed attempts: {}", peer_id, peer.failed_attempts);
            
            // Si excede el máximo, eliminarlo
            const MAX_FAILED_ATTEMPTS: u8 = 5;
            if peer.failed_attempts >= MAX_FAILED_ATTEMPTS {
                println!("❌ Peer {} exceeded max attempts, removing", peer_id);
                return self.remove_peer(peer_id);
            }
        }
        
        self.save_peers(&peers)
    }

    /// Resetear intentos fallidos de un peer (cuando conecta exitosamente)
    pub fn reset_peer_failed_attempts(&self, peer_id: &str) -> Result<()> {
        let mut peers = self.load_peers()?;
        
        if let Some(peer) = peers.iter_mut().find(|p| p.peer_id == peer_id) {
            peer.failed_attempts = 0;
            println!("✅ Reset failed attempts for peer: {}", peer_id);
        }
        
        self.save_peers(&peers)
    }

    // ========================================================================
    // PEER ID (IDENTIDAD LOCAL)
    // ========================================================================

    /// Cargar identidad local (par de llaves)
    pub fn load_peer_identity(&self) -> Result<Option<PeerIdToSave>> {
        let path = self.get_file_path("identity.json");
        
        if !path.exists() {
            println!("🔑 No identity file found");
            return Ok(None);
        }

        let content = fs::read_to_string(&path)
            .context("Failed to read identity file")?;
        
        let identity: PeerIdToSave = serde_json::from_str(&content)
            .context("Failed to parse identity file")?;
        
        println!("✅ Loaded identity");
        Ok(Some(identity))
    }

    /// Guardar identidad local
    pub fn save_peer_identity(&self, identity: &PeerIdToSave) -> Result<()> {
        let path = self.get_file_path("identity.json");
        let content = serde_json::to_string_pretty(identity)
            .context("Failed to serialize identity")?;
        
        fs::write(&path, content)
            .context("Failed to write identity file")?;
        
        println!("💾 Saved identity");
        Ok(())
    }

    // ========================================================================
    // CHANNELS
    // ========================================================================

    /// Cargar lista de canales/topics
    pub fn load_channels(&self) -> Result<Vec<ChannelInfoToSave>> {
        let path = self.get_file_path("channels.json");
        
        if !path.exists() {
            println!("📋 No channels file found, starting fresh");
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&path)
            .context("Failed to read channels file")?;
        
        let channels: Vec<ChannelInfoToSave> = serde_json::from_str(&content)
            .context("Failed to parse channels file")?;
        
        println!("✅ Loaded {} channels", channels.len());
        Ok(channels)
    }

    /// Guardar lista de canales
    pub fn save_channels(&self, channels: &[ChannelInfoToSave]) -> Result<()> {
        let path = self.get_file_path("channels.json");
        let content = serde_json::to_string_pretty(channels)
            .context("Failed to serialize channels")?;
        
        fs::write(&path, content)
            .context("Failed to write channels file")?;
        
        println!("💾 Saved {} channels", channels.len());
        Ok(())
    }

    /// Agregar o actualizar un canal
    pub fn add_or_update_channel(&self, channel: ChannelInfoToSave) -> Result<()> {
        let mut channels = self.load_channels()?;
        
        // Buscar si ya existe
        if let Some(existing) = channels.iter_mut().find(|c| c.uuid == channel.uuid) {
            *existing = channel;
            println!("🔄 Updated channel: {}", existing.topic);
        } else {
            println!("➕ Added new channel: {}", channel.topic);
            channels.push(channel);
        }
        
        self.save_channels(&channels)
    }

    /// Eliminar un canal
    pub fn remove_channel(&self, uuid: &str) -> Result<()> {
        let mut channels = self.load_channels()?;
        channels.retain(|c| c.uuid != uuid);
        
        println!("🗑️ Removed channel with UUID: {}", uuid);
        self.save_channels(&channels)
    }

    /// Actualizar último mensaje de un canal
    pub fn update_channel_last_message(&self, uuid: &str, message_uuid: String) -> Result<()> {
        let mut channels = self.load_channels()?;
        
        if let Some(channel) = channels.iter_mut().find(|c| c.uuid == uuid) {
            channel.last_message_uuid = Some(message_uuid.clone());
            println!("📝 Updated last message for channel: {}", channel.topic);
        }
        
        self.save_channels(&channels)
    }

    // ========================================================================
    // UTILIDADES
    // ========================================================================

    /// Obtener directorio base (útil para debugging o exportar)
    pub fn get_base_dir(&self) -> &PathBuf {
        &self.base_dir
    }

    /// Limpiar todos los datos (útil para reset)
    pub fn clear_all_data(&self) -> Result<()> {
        println!("⚠️ Clearing all data...");
        
        // Eliminar archivos individuales
        let files = ["peers.json", "identity.json", "channels.json"];
        
        for file in &files {
            let path = self.get_file_path(file);
            if path.exists() {
                fs::remove_file(&path)
                    .with_context(|| format!("Failed to remove {}", file))?;
                println!("🗑️ Removed {}", file);
            }
        }
        
        println!("✅ All data cleared");
        Ok(())
    }

    /// Exportar todos los datos a un directorio específico (backup)
    pub fn export_data(&self, export_dir: &PathBuf) -> Result<()> {
        fs::create_dir_all(export_dir)
            .context("Failed to create export directory")?;
        
        let files = ["peers.json", "identity.json", "channels.json"];
        
        for file in &files {
            let src = self.get_file_path(file);
            let dst = export_dir.join(file);
            
            if src.exists() {
                fs::copy(&src, &dst)
                    .with_context(|| format!("Failed to copy {}", file))?;
                println!("📤 Exported {}", file);
            }
        }
        
        println!("✅ Data exported to: {}", export_dir.display());
        Ok(())
    }

    /// Importar datos desde un directorio (restore)
    pub fn import_data(&self, import_dir: &PathBuf) -> Result<()> {
        let files = ["peers.json", "identity.json", "channels.json"];
        
        for file in &files {
            let src = import_dir.join(file);
            let dst = self.get_file_path(file);
            
            if src.exists() {
                fs::copy(&src, &dst)
                    .with_context(|| format!("Failed to import {}", file))?;
                println!("📥 Imported {}", file);
            }
        }
        
        println!("✅ Data imported from: {}", import_dir.display());
        Ok(())
    }
}

// ============================================================================
// IMPLEMENTACIÓN DE DEFAULT
// ============================================================================

impl Default for FileManager {
    fn default() -> Self {
        Self::new().expect("Failed to initialize FileManager")
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_manager_creation() {
        let fm = FileManager::new().unwrap();
        assert!(fm.get_base_dir().exists());
    }

    #[test]
    fn test_peers_crud() {
        let fm = FileManager::new().unwrap();
        
        // Agregar peer
        let peer = PeerInfoToSave {
            peer_id: "test123".to_string(),
            addresses: vec!["/ip4/127.0.0.1/tcp/4001".to_string()],
            failed_attempts: 0,
        };
        
        fm.add_or_update_peer(peer.clone()).unwrap();
        
        // Cargar y verificar
        let peers = fm.load_peers().unwrap();
        assert_eq!(peers.len(), 1);
        assert_eq!(peers[0].peer_id, "test123");
        
        // Eliminar
        fm.remove_peer("test123").unwrap();
        let peers = fm.load_peers().unwrap();
        assert_eq!(peers.len(), 0);
    }

    #[test]
    fn test_channels_crud() {
        let fm = FileManager::new().unwrap();
        
        // Agregar channel
        let channel = ChannelInfoToSave {
            topic: "general".to_string(),
            uuid: "abc-123".to_string(),
            last_message_uuid: None,
        };
        
        fm.add_or_update_channel(channel.clone()).unwrap();
        
        // Cargar y verificar
        let channels = fm.load_channels().unwrap();
        assert_eq!(channels.len(), 1);
        assert_eq!(channels[0].topic, "general");
        
        // Actualizar último mensaje
        fm.update_channel_last_message("abc-123", "msg-456".to_string()).unwrap();
        
        let channels = fm.load_channels().unwrap();
        assert_eq!(channels[0].last_message_uuid, Some("msg-456".to_string()));
        
        // Eliminar
        fm.remove_channel("abc-123").unwrap();
        let channels = fm.load_channels().unwrap();
        assert_eq!(channels.len(), 0);
    }
}