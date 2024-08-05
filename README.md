# flute-cli

Simple command line app based on examples of [Flute](https://github.com/ypo/flute) library.

Does not support multicast or setting advanced parameters like FEC for now.

There are pre-built releases for some platforms.

## Example

```
$ mkdir -p qqq

$ RUST_LOG=info flutecli recv 127.0.0.1:1234 qqq/

[INFO  flute_cli] Create FLUTE, write objects to "qqq/"

... (meanwhile `flutecli send 127.0.0.1:1234 Cargo.toml`)

[INFO  flute::receiver::multireceiver] Create FLUTE Receiver ReceiverEndpoint { endpoint: UDPEndpoint { source_address: None, destination_group_address: "127.0.0.1", port: 1234 }, tsi: 1 }
[INFO  flute::receiver::receiver] TSI=1 Attach FDT id 1
[INFO  flute::receiver::writer::objectwriterfs] Create destination "qqq/" "Cargo.toml" "qqq/Cargo.toml"
File Some("qqq/Cargo.toml") is completed !
```
