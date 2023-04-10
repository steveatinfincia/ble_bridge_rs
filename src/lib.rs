mod ble;

#[repr(C)]
pub struct BLEState {
    pub new_data_cb: extern "C" fn(*const std::ffi::c_char, SensorData, *mut std::ffi::c_void),
    pub userdata: *mut std::ffi::c_void,
}


#[repr(C)]
#[derive(Copy, Clone)]
pub struct CSwitchBotBotData {
    pub bluetooth_rssi: i16,
    pub battery: u8,
    pub state: bool,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CSwitchBotPlugData {
    pub bluetooth_rssi: i16,
    pub wifi_rssi: i16,
    pub state: bool,
    pub watts: i16,
    pub overload: bool,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CSwitchBotMeterData {
    pub bluetooth_rssi: i16,
    pub temperature: i32,
    pub humidity: u8,
    pub battery: u8,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CSwitchBotHumidifierData {
    pub bluetooth_rssi: i16,
    pub humidity: u8,
    pub state: bool,
    pub auto_mode: bool,
}


#[repr(C)]
#[derive(Debug)]
pub enum DeviceManufacturer {
    SwitchBot,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union DeviceModel {
    switchbot_device_model: switchbot::SwitchBotDeviceModel,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union DeviceData {
    switchbot_bot: CSwitchBotBotData,
    switchbot_plug: CSwitchBotPlugData,
    switchbot_meter: CSwitchBotMeterData,
    switchbot_humidifier: CSwitchBotHumidifierData
}

#[repr(C)]
pub struct SensorData {
    pub manufacturer: DeviceManufacturer,
    pub model: DeviceModel,
    pub device_data: DeviceData,
}

#[no_mangle]
pub extern "C" fn ble_bridge_run(state: &BLEState) -> i32 {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            match ble::run(state, |state, address, sensor_data| {
                let cstring = std::ffi::CString::new(address).unwrap();

                (state.new_data_cb)(cstring.as_ptr(), sensor_data, state.userdata);
            }).await {
                Ok(()) => {},
                Err(err) => {
                    println!("Error: {:?}", err);
                }
            }
        });
    
    0
}

