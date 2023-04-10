### ble_bridge_rs

A Rust FFI library for receiving Bluetooth LE data from SwitchBot devices.

This library is not listening passively (`bluez-async` doesn't support that at
the moment) so keep in mind that any connections to local devices may impact
their battery life.

This crate uses the [switchbot_rs](https://github.com/steveatinfincia/switchbot_rs)
crate to handle decoding the SwitchBot Bluetooth LE protocol, but it would otherwise
be usable for other manufacturers as long as you don't need to send any proprietary
commands to the device (which this crate does not attempt to handle at the moment,
even for SwitchBot devices).

This crate uses `tokio` and async Rust, but it is an implementation detail that
your code does not need to care about, as long as you are using it via the FFI
layer in `lib.rs`.

### Building

```
export RUSTFLAGS="--print=native-static-libs"
cargo build
```

This will generate:

* A dynamic library called `libble_bridge.so` in `target/debug/`
* A static library called `libble_bridge.a` in `target/debug/`
* A c++ header in `include/`

Using the staticlib will require you to handle linking any indirect dependencies,
but they will be printed on the screen during the build if you include `RUSTFLAGS`
as shown above. 

Make sure to add any displayed linker commands to your external build tool.

### API

There is only a single FFI function in this crate: `ble_bridge_run()`, and it
takes a single parameter, a `BLEState` struct.

In the struct, provide a callback in the form of a C function pointer, along
with a `void*` userdata pointer, which will be passed back to you in the callback
you provide.

If you are integrating with a UI of some kind you should run `ble_bridge_run()`
from a background thread and handle any necessary synchronization in your own
code.
