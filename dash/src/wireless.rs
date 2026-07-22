
//! Hotspot setup helpers.

use nmrs::{
    NetworkManager,
    builders::{WifiConnectionBuilder, WifiMode},
};

/// Start a hotspot connection on the provided Wi-Fi interface.
pub async fn start_hotspot(
    nm: &NetworkManager,
    ssid: &str,
    psk: &str,
    wifi_interface: &str,
) -> Result<(), String> {
    let settings = WifiConnectionBuilder::new(ssid)
        .wpa_psk(psk)
        .autoconnect(false)
        .mode(WifiMode::Ap)
        .ipv4_shared()
        .ipv6_ignore()
        .build();

    let (profile_path, active_path) = nm
        .add_and_activate_connection(settings, Some(wifi_interface), None)
        .await
        .map_err(|e| e.to_string())?;

    log::info!(
        "Started hotspot profile {} as active connection {}",
        profile_path,
        active_path
    );

    Ok(())
}

pub async fn get_wifi_interface(nmrs: &nmrs::NetworkManager) -> Option<nmrs::WifiDevice> {
    let dev = nmrs.list_wifi_devices().await.ok()?.into_iter().next()?;
    log::info!("Found wifi device {:?}", dev);
    Some(dev)
}
