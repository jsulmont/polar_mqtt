# rs-polar-mqtt

Code to support this blog post (see [here](intro.md) for a quick introduction).

## Option 1: Development with VS Code

This repository includes a development container configuration for VS Code. To use it:

1. Install [VS Code](https://code.visualstudio.com/)
2. Install the [Dev Containers extension](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.remote-containers)
3. Clone this repository
4. Open the repository folder in VS Code
5. When prompted, click "Reopen in Container" (or press F1, type "Dev Containers: Reopen in Container" and press Enter)

This will provide you with a development environment including all necessary tools for building and testing the project.

## Option 2: Running locally

Before building this project, ensure you have the following dependencies installed:

### Common Requirements
- Rust toolchain (stable)
- CMake (3.1 or higher)
- C++ compiler with C++17 (or above) support
- Eclipse [Paho MQTT C](https://github.com/eclipse-paho/paho.mqtt.c) Client Library

### macOS (via Homebrew)
```bash
brew install cmake
brew install paho-mqtt
```

### Linux Debian/Ubuntu
```bash
sudo apt update
sudo apt install build-essential cmake libssl-dev
sudo apt install libpaho-mqtt-dev
```

### Linux Arch Linux (requires AUR/yay)
```bash
sudo yay -S paho-mqtt-c-git
```

For other Linux distributions, please use the appropriate package manager.

## Building

This project uses Cargo with a custom build script (`build.rs`) to handle the C++ dependencies.

```bash
# Clone the repository
git clone https://github.com/jsulmont/rs-polar-mqtt
cd rs-polar-mqtt

# Build the project
cargo build
```

## Running the examples

There are currently 3 [examples](examples) which you should be able to run with crgo as usual:

```bash
cargo run --example mosquito_org

```
Note that `mosquito_org` will only work if `test.mosquitto.org` listen to port `1883` (it's sometime down).

## Testing

nope ... 😈



## Platform Support

| Platform | Status |
|----------|--------|
| macOS    | ✅     |
| Linux    | ✅     |



