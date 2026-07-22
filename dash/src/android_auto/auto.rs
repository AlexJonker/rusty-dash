// Copied from https://github.com/uglyoldbob/android-auto/
use android_auto::{AndroidAutoMainTrait, HeadUnitInfo, VideoConfiguration};
use bluetooth_rust::MessageToBluetoothHost;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use openh264::formats::YUVSource;
use ringbuf::traits::Producer;
use slint::ComponentHandle;
use std::{
    collections::HashSet,
    sync::Arc,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};
use tokio::sync::Mutex;

use crate::AppWindow;
use crate::android_auto::wireless;

type AudioProducer = ringbuf::HeapProd<i16>;

struct AndroidAutoInner {
    relay: Option<tokio::task::JoinHandle<()>>,
    connected: bool,
    send: tokio::sync::mpsc::Sender<MessageFromAsync>,
    arecv: Option<tokio::sync::mpsc::Receiver<android_auto::SendableAndroidAutoMessage>>,
    android_send: tokio::sync::mpsc::Sender<android_auto::SendableAndroidAutoMessage>,
    audio_input: Option<cpal::Device>,
    media_stream: Option<(AudioProducer, cpal::Stream)>,
    sys_stream: Option<(AudioProducer, cpal::Stream)>,
    speech_stream: Option<(AudioProducer, cpal::Stream)>,
    input_stream: Option<cpal::Stream>,
}

#[derive(Clone)]
struct AndroidAuto {
    inner: Arc<Mutex<AndroidAutoInner>>,
    config: VideoConfiguration,
    blue: Option<android_auto::BluetoothInformation>,
    bluetooth: Option<Arc<bluetooth_rust::BluetoothAdapter>>,
    network: Option<Arc<android_auto::NetworkInformation>>,
    sensors: android_auto::SensorInformation,
    input_config: android_auto::InputConfiguration,
}

enum MessageFromAsync {
    VideoData {
        data: Vec<u8>,
        _timestamp: Option<u64>,
    },
    Connected,
    Disconnected,
    ExitContainer,
}

#[allow(dead_code)]
enum MessageToAsync {
    AndroidAutoMessage(android_auto::SendableAndroidAutoMessage),
}

#[async_trait::async_trait]
impl android_auto::AndroidAutoVideoChannelTrait for AndroidAuto {
    async fn receive_video(&self, data: Vec<u8>, timestamp: Option<u64>) {
        let i = self.inner.lock().await;
        let _ = i
            .send
            .send(MessageFromAsync::VideoData {
                data,
                _timestamp: timestamp,
            })
            .await;
    }

    async fn setup_video(&self) -> Result<(), ()> {
        Ok(())
    }

    async fn teardown_video(&self) {}

    async fn wait_for_focus(&self) {}

    async fn set_focus(&self, _focus: bool) {}

    fn retrieve_video_configuration(&self) -> &VideoConfiguration {
        &self.config
    }
}

#[async_trait::async_trait]
impl android_auto::AndroidAutoSensorTrait for AndroidAuto {
    fn get_supported_sensors(&self) -> &android_auto::SensorInformation {
        &self.sensors
    }

    async fn start_sensor(&self, stype: android_auto::Wifi::sensor_type::Enum) -> Result<(), ()> {
        if self.sensors.sensors.contains(&stype) {
            let mut m3 = android_auto::Wifi::SensorEventIndication::new();
            match stype {
                android_auto::Wifi::sensor_type::Enum::DRIVING_STATUS => {
                    let mut ds = android_auto::Wifi::DrivingStatus::new();
                    ds.set_status(android_auto::Wifi::DrivingStatusEnum::UNRESTRICTED as i32);
                    m3.driving_status.push(ds);
                }
                android_auto::Wifi::sensor_type::Enum::NIGHT_DATA => {
                    let mut ds = android_auto::Wifi::NightMode::new();
                    ds.set_is_night(false);
                    m3.night_mode.push(ds);
                }
                _ => {
                    todo!();
                }
            }
            let s = self.inner.lock().await;
            let m = android_auto::AndroidAutoMessage::Sensor(m3);
            s.android_send.send(m.sendable()).await.map_err(|_| ())?;
            Ok(())
        } else {
            Err(())
        }
    }
}

#[async_trait::async_trait]
impl android_auto::AndroidAutoAudioOutputTrait for AndroidAuto {
    async fn open_output_channel(&self, _t: android_auto::AudioChannelType) -> Result<(), ()> {
        Ok(())
    }

    async fn close_output_channel(&self, _t: android_auto::AudioChannelType) -> Result<(), ()> {
        Ok(())
    }

    async fn receive_output_audio(&self, t: android_auto::AudioChannelType, data: Vec<u8>) {
        let mut s = self.inner.lock().await;
        let r2: Vec<i16> = data
            .as_chunks::<2>().0.iter()
            .map(|v| i16::from_le_bytes([v[0], v[1]]))
            .collect();
        match t {
            android_auto::AudioChannelType::Media => {
                s.media_stream.as_mut().map(|m| m.0.push_slice(&r2));
            }
            android_auto::AudioChannelType::System => {
                s.sys_stream.as_mut().map(|m| m.0.push_slice(&r2));
            }
            android_auto::AudioChannelType::Speech => {
                s.speech_stream.as_mut().map(|m| m.0.push_slice(&r2));
            }
        }
    }

    async fn start_output_audio(&self, t: android_auto::AudioChannelType) {
        let s = self.inner.lock().await;
        match t {
            android_auto::AudioChannelType::Media => {
                s.media_stream.as_ref().map(|m| m.1.play());
            }
            android_auto::AudioChannelType::System => {
                s.sys_stream.as_ref().map(|m| m.1.play());
            }
            android_auto::AudioChannelType::Speech => {
                s.speech_stream.as_ref().map(|m| m.1.play());
            }
        }
    }

    async fn stop_output_audio(&self, t: android_auto::AudioChannelType) {
        let s = self.inner.lock().await;
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        match t {
            android_auto::AudioChannelType::Media => {
                s.media_stream.as_ref().map(|m| m.1.pause());
            }
            android_auto::AudioChannelType::System => {
                s.sys_stream.as_ref().map(|m| m.1.pause());
            }
            android_auto::AudioChannelType::Speech => {
                s.speech_stream.as_ref().map(|m| m.1.pause());
            }
        }
    }
}

#[async_trait::async_trait]
impl android_auto::AndroidAutoInputChannelTrait for AndroidAuto {
    async fn binding_request(&self, _code: u32) -> Result<(), ()> {
        Ok(())
    }

    fn retrieve_input_configuration(&self) -> &android_auto::InputConfiguration {
        &self.input_config
    }
}

#[async_trait::async_trait]
impl android_auto::AndroidAutoAudioInputTrait for AndroidAuto {
    async fn open_input_channel(&self) -> Result<(), ()> {
        let mut s = self.inner.lock().await;
        let config = cpal::StreamConfig {
            channels: 1,
            sample_rate: 16000,
            buffer_size: cpal::BufferSize::Default,
        };
        if let Some(ai) = &s.audio_input {
            let android_send = s.android_send.clone();
            if let Ok(str) = ai.build_input_stream(
                config,
                move |data: &[i16], _| {
                    let bytes: Vec<u8> = data.iter().flat_map(|s| s.to_le_bytes()).collect();
                    let timestamp = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_micros()
                        .try_into()
                        .unwrap_or(u64::MAX);
                    let msg = android_auto::AndroidAutoMessage::Audio(Some(timestamp), bytes);
                    if let Err(e) = android_send.try_send(msg.sendable()) {
                        log::warn!("Dropped audio input frame: {e:?}");
                    }
                },
                |err| log::error!("Audio input error: {err:?}"),
                None,
            ) {
                let _ = str.play();
                s.input_stream = Some(str);
            } else {
                log::error!("Failed to open input channel stream");
            }
        }
        Ok(())
    }
    async fn close_input_channel(&self) -> Result<(), ()> {
        let mut s = self.inner.lock().await;
        s.input_stream.take();
        Ok(())
    }
    async fn start_input_audio(&self) {}

    async fn audio_input_ack(&self, chan: u8, ack: android_auto::Wifi::AVMediaAckIndication) {
        log::info!("Ack audio input for chan {chan} {ack:?}");
    }

    async fn stop_input_audio(&self) {
        log::error!("Stop audio input channel");
        let mut s = self.inner.lock().await;
        s.input_stream.take();
    }
}

#[async_trait::async_trait]
impl android_auto::AndroidAutoWiredTrait for AndroidAuto {}

#[async_trait::async_trait]
impl android_auto::AndroidAutoWirelessTrait for AndroidAuto {
    async fn setup_bluetooth_profile(
        &self,
        suggestions: &bluetooth_rust::BluetoothRfcommProfileSettings,
    ) -> Result<bluetooth_rust::BluetoothRfcommProfile, String> {
        match &self.bluetooth {
            Some(bt) => bt.register_rfcomm_profile(suggestions.clone()).await,
            None => Err("Bluetooth not available".to_string()),
        }
    }

    fn get_wifi_details(&self) -> android_auto::NetworkInformation {
        self.network
            .as_ref()
            .expect("get_wifi_details called without network info")
            .as_ref()
            .clone()
    }
}

#[async_trait::async_trait]
impl android_auto::AndroidAutoBluetoothTrait for AndroidAuto {
    async fn do_stuff(&self) {}

    fn get_config(&self) -> &android_auto::BluetoothInformation {
        self.blue
            .as_ref()
            .expect("get_config called without bluetooth info")
    }
}

#[async_trait::async_trait]
impl android_auto::AndroidAutoMainTrait for AndroidAuto {
    async fn connect(&self) {
        let mut i = self.inner.lock().await;
        let _ = i.send.send(MessageFromAsync::Connected).await;
        log::info!("Android auto connected");
        i.connected = true;
    }

    async fn disconnect(&self) {
        let mut s = self.inner.lock().await;
        let _ = s.send.send(MessageFromAsync::Disconnected).await;
        log::info!("Android auto disconnected");
        s.connected = false;
    }

    async fn get_receiver(
        &self,
    ) -> Option<tokio::sync::mpsc::Receiver<android_auto::SendableAndroidAutoMessage>> {
        let mut s = self.inner.lock().await;
        s.arecv.take()
    }

    fn supports_bluetooth(&self) -> Option<&dyn android_auto::AndroidAutoBluetoothTrait> {
        self.blue.as_ref()?;
        Some(self)
    }

    fn supports_wireless(&self) -> Option<Arc<dyn android_auto::AndroidAutoWirelessTrait>> {
        self.network.as_ref()?;
        Some(Arc::new(self.clone()))
    }

    fn supports_wired(&self) -> Option<Arc<dyn android_auto::AndroidAutoWiredTrait>> {
        Some(Arc::new(self.clone()))
    }
}

fn try_build_output_stream(
    device: &cpal::Device,
    configs: &[cpal::SupportedStreamConfigRange],
    rate: u32,
    channels: u16,
    ring_size: usize,
) -> Option<(AudioProducer, cpal::Stream)> {
    let cfg = (*configs.iter().find(|c| {
        c.channels() == channels
            && c.sample_format() == cpal::SampleFormat::I16
            && c.min_sample_rate() <= rate
            && c.max_sample_rate() >= rate
    })?)
    .try_with_sample_rate(rate)?;

    let rb = ringbuf::HeapRb::new(ring_size);
    let (producer, mut consumer) = ringbuf::traits::Split::split(rb);
    let stream = device
        .build_output_stream(
            cfg.config(),
            move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                let mut index = 0;
                while index < data.len() {
                    let c = ringbuf::traits::Consumer::pop_slice(&mut consumer, &mut data[index..]);
                    if c == 0 {
                        break;
                    }
                    index += c;
                }
            },
            |err| log::error!("Error in audio output: {err:?}"),
            None,
        )
        .ok()?;
    Some((producer, stream))
}

impl AndroidAuto {
    fn new(
        mut recv: tokio::sync::mpsc::Receiver<MessageToAsync>,
        send: tokio::sync::mpsc::Sender<MessageFromAsync>,
        bluetooth: Option<Arc<bluetooth_rust::BluetoothAdapter>>,
        blue_address: Option<String>,
        network: Option<android_auto::NetworkInformation>,
        android_recv: tokio::sync::mpsc::Receiver<android_auto::SendableAndroidAutoMessage>,
        android_send: tokio::sync::mpsc::Sender<android_auto::SendableAndroidAutoMessage>,
    ) -> Self {
        let mut sensors = HashSet::new();
        sensors.insert(android_auto::Wifi::sensor_type::Enum::DRIVING_STATUS);
        sensors.insert(android_auto::Wifi::sensor_type::Enum::NIGHT_DATA);

        let android_send2 = android_send.clone();
        let relay = tokio::spawn(async move {
            while let Some(MessageToAsync::AndroidAutoMessage(msg)) = recv.recv().await {
                if let Err(e) = android_send2.send(msg).await {
                    log::error!("Error relaying info {e:?}");
                    break;
                }
            }
        });

        let host = cpal::default_host();
        let ai = host.default_input_device();
        let (media_stream, sys_stream, speech_stream) = host
            .default_output_device()
            .and_then(|dev| {
                let cfgs: Vec<_> = dev.supported_output_configs().ok()?.collect();
                let media = try_build_output_stream(&dev, &cfgs, 48000, 2, 48000);
                let sys = try_build_output_stream(&dev, &cfgs, 16000, 1, 16000);
                let speech = try_build_output_stream(&dev, &cfgs, 16000, 1, 16000);
                Some((media, sys, speech))
            })
            .unwrap_or((None, None, None));

        Self {
            inner: Arc::new(Mutex::new(AndroidAutoInner {
                relay: Some(relay),
                connected: false,
                send,
                arecv: Some(android_recv),
                android_send,
                audio_input: ai,
                media_stream,
                sys_stream,
                speech_stream,
                input_stream: None,
            })),
            bluetooth,
            network: network.map(Arc::new),
            blue: blue_address.map(|addr| android_auto::BluetoothInformation { address: addr }),
            config: VideoConfiguration {
                resolution: android_auto::Wifi::video_resolution::Enum::_480p,
                fps: android_auto::Wifi::video_fps::Enum::_30,
                dpi: 111,
            },
            sensors: android_auto::SensorInformation { sensors },
            input_config: android_auto::InputConfiguration {
                keycodes: vec![1, 2, 3, 4, 5],
                touchscreen: Some((800, 480)),
            },
        }
    }

    async fn start_android_auto(
        self,
        config: android_auto::AndroidAutoConfiguration,
        setup: android_auto::AndroidAutoSetup,
    ) -> Result<(), String> {
        let mut joinset = tokio::task::JoinSet::new();
        let relay = self.inner.lock().await.relay.take();
        let result = Box::new(self).run(config, &mut joinset, &setup).await;
        joinset.join_all().await;
        if let Some(r) = relay {
            r.abort()
        }
        result
    }
}

struct AndroidAutoContainer {
    thread: Option<std::thread::JoinHandle<Result<(), String>>>,
    recv: tokio::sync::mpsc::Receiver<MessageFromAsync>,
    #[allow(dead_code)]
    send: tokio::sync::mpsc::Sender<MessageToAsync>,
    kill: Option<tokio::sync::oneshot::Sender<()>>,
}

impl AndroidAutoContainer {
    fn new(setup: android_auto::AndroidAutoSetup) -> Self {
        let to_async = tokio::sync::mpsc::channel(50);
        let from_async = tokio::sync::mpsc::channel(50);
        let (kill_tx, kill_rx) = tokio::sync::oneshot::channel::<()>();
        let send_exit = from_async.0.clone();

        let thread = std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("Failed to build Tokio runtime");

            let wireless_enabled = std::env::var("ENABLE_WIRELESS")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(false);

            let r = rt.block_on(async {
                let (bluetooth, blue_address, network) = if wireless_enabled {
                    let wifi = nmrs::NetworkManager::new().await.expect("WiFi not found");
                    let wifi_dev = wireless::get_wifi_interface(&wifi)
                        .await
                        .expect("No wifi device found");
                    let hotspot_ssid = "Hotspot".to_string();
                    let hotspot_psk = "qwertyuiop".to_string();
                    wireless::start_hotspot(
                        &wifi,
                        &hotspot_ssid,
                        &hotspot_psk,
                        &wifi_dev.interface,
                    )
                    .await
                    .expect("Failed to start wifi hotspot");
                    let (mut bluechan, bt_adapter) = {
                        let bluechan = tokio::sync::mpsc::channel(5);
                        let mut builder = bluetooth_rust::BluetoothAdapterBuilder::new();
                        builder.with_sender(bluechan.0);
                        let bt = Arc::new(builder.build().await.expect("Could not open bluetooth"));
                        (bluechan.1, bt)
                    };
                    bt_adapter
                        .set_discoverable(true)
                        .await
                        .expect("Failed to make bluetooth discoverable");
                    tokio::spawn(async move {
                        loop {
                            if let Some(m) = bluechan.recv().await {
                                match m {
                                    MessageToBluetoothHost::DisplayPasskey(a, sender) => {
                                        log::info!("Passkey is {}", a);
                                        let _ = sender
                                            .send(bluetooth_rust::ResponseToPasskey::Yes)
                                            .await;
                                    }
                                    MessageToBluetoothHost::ConfirmPasskey(a, sender) => {
                                        log::info!("Passkey is confirmed {}", a);
                                        let _ = sender
                                            .send(bluetooth_rust::ResponseToPasskey::Yes)
                                            .await;
                                    }
                                    MessageToBluetoothHost::CancelDisplayPasskey => {
                                        log::info!("Cancel show passkey");
                                    }
                                }
                            }
                        }
                    });
                    let blue_addresses: Vec<bluetooth_rust::BluetoothAdapterAddress> =
                        bt_adapter.addresses().await;
                    let bt_address = blue_addresses
                        .first()
                        .map(|b| match b {
                            bluetooth_rust::BluetoothAdapterAddress::String(s) => s.to_owned(),
                            bluetooth_rust::BluetoothAdapterAddress::Byte(b) => {
                                format!(
                                    "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                                    b[0], b[1], b[2], b[3], b[4], b[5]
                                )
                            }
                        })
                        .expect("No bluetooth hardware found");

                    let net = android_auto::NetworkInformation {
                        ssid: hotspot_ssid,
                        psk: hotspot_psk,
                        mac_addr: wifi_dev.hw_address,
                        ip: "10.42.0.1".to_string(),
                        port: 5277,
                        security_mode: android_auto::Bluetooth::SecurityMode::WPA2_PERSONAL,
                        ap_type: android_auto::Bluetooth::AccessPointType::STATIC,
                    };
                    (Some(bt_adapter), Some(bt_address), Some(net))
                } else {
                    log::warn!("Wireless disabled by ENABLE_WIRELESS in /storage/settings.conf");
                    (None, None, None)
                };

                let aauto = tokio::sync::mpsc::channel(50);
                let aa = AndroidAuto::new(
                    to_async.1,
                    from_async.0,
                    bluetooth,
                    blue_address,
                    network,
                    aauto.1,
                    aauto.0,
                );
                let config = android_auto::AndroidAutoConfiguration {
                    unit: HeadUnitInfo {
                        name: "Miata".to_string(),
                        car_model: "Miata".to_string(),
                        car_year: "1990".to_string(),
                        car_serial: "67".to_string(),
                        left_hand: false,
                        head_manufacturer: "Mazda".to_string(),
                        head_model: "rusty-dash".to_string(),
                        sw_build: "37".to_string(),
                        sw_version: "1.2.3".to_string(),
                        native_media: true,
                        hide_clock: Some(true),
                    },
                    custom_certificate: None,
                };
                tokio::select! {
                    _ = aa.start_android_auto(config, setup) => {}
                    _ = kill_rx => {}
                }
                Ok::<(), String>(())
            });

            send_exit
                .blocking_send(MessageFromAsync::ExitContainer)
                .map_err(|e| e.to_string())?;
            r
        });

        Self {
            thread: Some(thread),
            recv: from_async.1,
            send: to_async.0,
            kill: Some(kill_tx),
        }
    }
}

impl Drop for AndroidAutoContainer {
    fn drop(&mut self) {
        let _ = self.kill.take().map(|s| s.send(()));
        self.thread.take().map(std::thread::JoinHandle::join);
    }
}

pub struct AndroidAutoController {
    thread: Option<std::thread::JoinHandle<()>>,
    should_exit: Arc<AtomicBool>,
    send: tokio::sync::mpsc::Sender<MessageToAsync>,
}

impl Clone for AndroidAutoController {
    fn clone(&self) -> Self {
        Self {
            thread: None,
            should_exit: self.should_exit.clone(),
            send: self.send.clone(),
        }
    }
}

impl AndroidAutoController {
    pub fn new(ui: &AppWindow) -> Self {
        let ui_weak = ui.as_weak();
        let setup = android_auto::setup();
        let should_exit = Arc::new(AtomicBool::new(false));
        let should_exit_clone = should_exit.clone();

        let mut container = AndroidAutoContainer::new(setup);
        let send = container.send.clone();

        let thread = std::thread::spawn(move || {
            let mut decoder = openh264::decoder::Decoder::new().unwrap();

            loop {
                if should_exit_clone.load(Ordering::SeqCst) {
                    break;
                }
                match container.recv.try_recv() {
                    Ok(MessageFromAsync::ExitContainer) => {
                        container = AndroidAutoContainer::new(setup);
                    }
                    Ok(MessageFromAsync::Connected) => {}
                    Ok(MessageFromAsync::Disconnected) => {
                        let _ = decoder.flush_remaining();
                        set_ui_frame_data(&ui_weak, None);
                    }
                    Ok(MessageFromAsync::VideoData { data, _timestamp }) => {
                        update_frame_from_video(&ui_weak, &mut decoder, &data);
                    }
                    Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {
                        std::thread::sleep(Duration::from_millis(8));
                    }
                    Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                        container = AndroidAutoContainer::new(setup);
                    }
                }
            }
        });

        let handle = Self {
            thread: Some(thread),
            should_exit,
            send,
        };

        let handle_clone = handle.clone();
        ui.on_handle_android_auto_touch(move |x, y, action| {
            handle_clone.send_touch_event(x, y, action);
        });

        handle
    }

    pub fn send_touch_event(&self, x: i32, y: i32, action: i32) {
        let mut i_event = android_auto::Wifi::InputEventIndication::new();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_micros() as u64;
        i_event.set_timestamp(timestamp);

        let mut te = android_auto::Wifi::TouchEvent::new();
        let mut tl = android_auto::Wifi::TouchLocation::new();
        tl.set_x(x as u32);
        tl.set_y(y as u32);
        tl.set_pointer_id(0);
        te.touch_location.push(tl);

        te.set_touch_action(match action {
            2 => android_auto::Wifi::touch_action::Enum::DRAG,
            5 => android_auto::Wifi::touch_action::Enum::POINTER_DOWN,
            6 => android_auto::Wifi::touch_action::Enum::POINTER_UP,
            _ => android_auto::Wifi::touch_action::Enum::POINTER_DOWN,
        });

        i_event.touch_event = android_auto::protobuf::MessageField::some(te);

        let msg = android_auto::AndroidAutoMessage::Input(i_event);

        if let Err(e) = self
            .send
            .try_send(MessageToAsync::AndroidAutoMessage(msg.sendable()))
        {
            log::error!("Failed to send touch event: {e:?}");
        }
    }
}

impl Drop for AndroidAutoController {
    fn drop(&mut self) {
        self.should_exit.store(true, Ordering::SeqCst);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn update_frame_from_video(
    ui_weak: &slint::Weak<AppWindow>,
    decoder: &mut openh264::decoder::Decoder,
    data: &[u8],
) {
    let units: Vec<&[u8]> = openh264::nal_units(data).collect();
    for p in &units {
        match decoder.decode(p) {
            Err(e) => log::error!("Failed to decode android auto video {e:?}"),
            Ok(None) => {}
            Ok(Some(image)) => {
                let mut rgb_raw = vec![0; image.rgb8_len()];
                image.write_rgb8(&mut rgb_raw);
                let (w, h) = image.dimensions_uv();
                let (width, height) = (w * 2, h * 2);
                let rgba: Vec<u8> = rgb_raw
                    .as_chunks::<3>().0.iter()
                    .flat_map(|c| [c[0], c[1], c[2], 255])
                    .collect();
                set_ui_frame_data(
                    ui_weak,
                    Some(FrameData {
                        width: width as u32,
                        height: height as u32,
                        rgba,
                    }),
                );
            }
        }
    }
}

struct FrameData {
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

fn set_ui_frame_data(ui_weak: &slint::Weak<AppWindow>, frame: Option<FrameData>) {
    let ui_weak = ui_weak.clone();
    let _ = slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui_weak.upgrade() {
            let image = match frame {
                Some(f) => {
                    let mut buffer =
                        slint::SharedPixelBuffer::<slint::Rgba8Pixel>::new(f.width, f.height);
                    buffer.make_mut_bytes().copy_from_slice(&f.rgba);
                    slint::Image::from_rgba8(buffer)
                }
                None => slint::Image::default(),
            };
            ui.set_android_auto_frame(image);
        }
    });
}
