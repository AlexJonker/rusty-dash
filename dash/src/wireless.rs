use std::collections::HashMap;
use zbus::{Connection, proxy};
use zvariant::{OwnedObjectPath, OwnedValue};

#[proxy(
    interface = "org.freedesktop.NetworkManager",
    default_service = "org.freedesktop.NetworkManager",
    default_path = "/org/freedesktop/NetworkManager"
)]
trait NmProxy {
    fn add_and_activate_connection(
        &self,
        connection: &HashMap<String, HashMap<String, OwnedValue>>,
        device: &OwnedObjectPath,
        specific_object: &OwnedObjectPath,
    ) -> zbus::Result<(OwnedObjectPath, OwnedObjectPath)>;
}

fn to_owned_settings(
    input: HashMap<&str, HashMap<&str, zvariant::Value<'_>>>,
) -> HashMap<String, HashMap<String, OwnedValue>> {
    input
        .into_iter()
        .map(|(section, props)| {
            let owned_props = props
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.try_to_owned().unwrap()))
                .collect();
            (section.to_string(), owned_props)
        })
        .collect()
}

pub async fn start_hotspot(ssid: String, psk: String, wifi_dev_path: &str) -> Result<(), String> {
    let hotspot = nmrs::builders::WifiConnectionBuilder::new(&ssid)
        .wpa_psk(&psk)
        .autoconnect(false)
        .mode(nmrs::builders::WifiMode::Ap)
        .build();
    let settings = to_owned_settings(hotspot);
    let dbus = Connection::system().await.map_err(|e| e.to_string())?;
    let wifi_device = OwnedObjectPath::try_from(wifi_dev_path).map_err(|e| e.to_string())?;
    let any: OwnedObjectPath = OwnedObjectPath::try_from("/").unwrap();
    let nm = NmProxyProxy::new(&dbus).await.map_err(|e| e.to_string())?;
    nm.add_and_activate_connection(&settings, &wifi_device, &any)
        .await
        .map_err(|e| e.to_string())?;
    log::info!("Hotspot '{}' started successfully", ssid);
    Ok(())
}

pub async fn get_wifi_interface(nmrs_client: &nmrs::NetworkManager) -> Option<nmrs::Device> {
    if let Ok(devs) = nmrs_client.list_wireless_devices().await {
        for dev in devs {
            if dev.device_type == nmrs::DeviceType::Wifi {
                log::info!("Found wifi device {:?}", dev);
                return Some(dev);
            }
        }
    }
    None
}
