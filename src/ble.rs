use bluez_async::{BluetoothSession, DiscoveryFilter};

#[allow(unused_imports)]
use uuid::{Uuid};
use futures::stream::StreamExt;
use std::error::Error;

use switchbot::model::{SwitchBotData};

use crate::BLEState;
use crate::SensorData;
use crate::DeviceManufacturer;
use crate::DeviceModel;
use crate::DeviceData;
use crate::CSwitchBotBotData;
use crate::CSwitchBotMeterData;
use crate::CSwitchBotPlugData;
use crate::CSwitchBotHumidifierData;

pub async fn run(state: &BLEState,
                 cb: fn(state: &BLEState,
                        address: String,
                        sensor_data: SensorData)) -> Result<(), Box<dyn Error>> {
    let (_, session) = BluetoothSession::new().await?;
    let mut events = session.event_stream().await?;

    session.start_discovery_with_filter(&DiscoveryFilter {
        duplicate_data: Some(true),
        discoverable: Some(false),
        transport: Some(bluez_async::Transport::Auto),
        ..DiscoveryFilter::default()
    }).await?;

    while let Some(event) = events.next().await {
        match event {
            bluez_async::BluetoothEvent::Device { id, event } => {
                let Ok(device_info) = session.get_device_info(&id).await else {
                    continue;
                };
                
                let mac_address = device_info.mac_address;

                match event {
                    bluez_async::DeviceEvent::Discovered => {}
                    bluez_async::DeviceEvent::Connected { connected: _ } => {}
                    bluez_async::DeviceEvent::ManufacturerData { manufacturer_data: _ } => {}
                    bluez_async::DeviceEvent::Rssi { rssi: _ } => {}
                    bluez_async::DeviceEvent::Services { services: _} => {}
                    bluez_async::DeviceEvent::ServicesResolved => {}
                    bluez_async::DeviceEvent::ServiceData { service_data } => {
                        let Some(service_data) = service_data.values().last() else {
                            continue;
                        };

                        let manufacturer_data = device_info.manufacturer_data.values().last();

                        let (Some(model),
                             Some(switchbot_data)) = switchbot::protocol::decode_data(service_data,
                                                                                      manufacturer_data.map(Vec::as_slice))
                        else {
                            continue;
                        };

                        let bluetooth_rssi = match device_info.rssi { 
                            Some(rssi) => rssi,
                            None => 0,
                        };

                        let mut device_data: DeviceData;

                        match switchbot_data {
                            SwitchBotData::Bot { battery,
                                                 state } => {
                                device_data.switchbot_bot = CSwitchBotBotData {
                                    bluetooth_rssi,
                                    state,
                                    battery,
                                }
                            }
                            SwitchBotData::Meter { battery,
                                                   temperature,
                                                   humidity } => {
                                device_data.switchbot_meter = CSwitchBotMeterData {
                                    bluetooth_rssi,
                                    battery,
                                    temperature,
                                    humidity,
                                }
                            }
                            SwitchBotData::Plug { wifi_rssi,
                                                  state,
                                                  watts,
                                                  overload } => {
                                device_data.switchbot_plug = CSwitchBotPlugData {
                                    bluetooth_rssi,
                                    wifi_rssi,
                                    state,
                                    watts,
                                    overload,
                                }
                            }
                            SwitchBotData::Humidifier { state,
                                                        humidity,
                                                        auto_mode } => {
                                device_data.switchbot_humidifier= CSwitchBotHumidifierData {
                                    bluetooth_rssi,
                                    state,
                                    humidity,
                                    auto_mode
                                }
                            }
                        }

                        let sensor_data = SensorData {
                            manufacturer: DeviceManufacturer::SwitchBot,
                            model: DeviceModel { 
                                switchbot_device_model: model.clone()
                            },
                            device_data,
                        };

                        cb(state, mac_address.to_string(), sensor_data);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    Ok(())
}
