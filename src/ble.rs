/*
 * This module is a demonstration of using bluez-async to receive
 * Bluetooth LE advertisements, and using the switchbot_rs crate to
 * decode them.
 * 
 * Ideally you would want to actively connect to newly discovered devices,
 * discover the SwitchBot primary service UUID (if the device provides it),
 * and only then attempt to parse the data the device advertises.
 * 
 * However this crate does not currently do that, it parses the data directly
 * and attempts to avoid false positives, which works out well in practice
 * for informal testing.
 */
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

/*
 * If you find yourself needing to call this function directly, just copy
 * the function into your own code and modify it as needed, it's only 100
 * lines and you will likely need to modify it anyway.
 */
pub async fn run(state: &BLEState,
                 cb: fn(state: &BLEState,
                        address: String,
                        sensor_data: SensorData)) -> Result<(), Box<dyn Error>> {
    let (_, session) = BluetoothSession::new().await?;
    let mut events = session.event_stream().await?;

    /*
     * This is intentionally avoiding a service UUID filter, not
     * all SwitchBot devices advertise one so filtering would
     * prevent receving anything from those devices.
     */
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

                        /*
                         * This converts from the Rust enum type in the switchbot_rs crate,
                         * to a FFI equivalent structs from this crate, while adding bluetooth
                         * rssi as a convenience.
                         */
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
