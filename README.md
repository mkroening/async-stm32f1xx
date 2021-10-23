[![Continuous integration](https://github.com/mwkroening/async-stm32f1xx/workflows/Continuous%20integration/badge.svg?branch=main)](https://github.com/mwkroening/async-stm32f1xx/actions?query=workflow%3A%22Continuous+integration%22+branch%3Amain) [![Version](https://img.shields.io/crates/v/async-stm32f1xx)](https://crates.io/crates/async-stm32f1xx) [![Documentation](https://docs.rs/async-stm32f1xx/badge.svg)](https://docs.rs/async-stm32f1xx) [![License](https://img.shields.io/crates/l/async-stm32f1xx)](#license)

# async-stm32f1xx

Abstractions for asynchronous programming on the STM32F1xx family of microcontrollers.

This project provides [`futures-rs`](https://github.com/rust-lang/futures-rs)-based abstractions for asynchronous programming with peripherals from [`stm32f1xx-hal`](https://github.com/stm32-rs/stm32f1xx-hal).
It started as an effort to port the [examples from `async-on-embedded`](https://github.com/rust-embedded-community/async-on-embedded/tree/master/nrf52/examples) to the [Blue Pill (`STM32F103C8T6`)](https://stm32-base.org/boards/STM32F103C8T6-Blue-Pill.html) as part of a bachelor's thesis.
The library is independent of any particular executor, but the [examples](examples) use the [`async-embedded`](https://github.com/rust-embedded-community/async-on-embedded/tree/master/async-embedded) runtime.

## [Examples](examples)

Most of `async-on-embedded`'s examples have been successfully ported to this project.

### Requirements

* [probe-run](https://github.com/knurling-rs/probe-run)

### Adjusting to your hardware

The [memory region information](memory.x) included in this repository matches the Blue Pill (`STM32F103C8T6`).
You may need to adjust it according to your hardware.
For more information see [`cortex-m-quickstart`](https://github.com/rust-embedded/cortex-m-quickstart).

### Running

You can run the example via cargo:

``` 
$ cargo run --example <NAME> [--release]
```

## License

This project is licensed under either of

* [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0) ([LICENSE-APACHE](LICENSE-APACHE))

* [MIT License](https://opensource.org/licenses/MIT) ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Contributing

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in `async-stm32f1xx` by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
