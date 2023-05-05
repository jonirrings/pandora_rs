## Usage
**Build the project**
```shell
    cargo build --release
```
**Flash to target**
```shell
    cargo flash --chip STM32L475VETx --connect-under-reset --release
````
**If updating linker script or things Cargo might not notice, it can be helpful to follow up with**
```shell
    cargo clean
```
**strip and st-flash **
```shell
    rust-objcopy --binary-architecture=thumbv7m pandora_rs --strip-all -O binary pandora.bin
    st-flash --reset write pandora.bin 0x8000000
```
