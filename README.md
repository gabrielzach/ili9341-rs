# `ili9341-buffered`

A fork of *ili9341-rs* to adapt it to my use case.

**Changes**:

- Adapt to the M5Stack platform (flip display, fix inverted colors)
- Use embedded-graphics 0.5.2 instead of 0.6 (compatibility issues with old Rust version in my ESP32 dev environment)
- Use a simple framebuffer instead of drawing each pixel individually (vast performance improvement)
- Modified the SPI *write* method so that data is written in chunks of configurable size (due to limits of the underlying SPI implementation)

**Note**:

The memory block to use as framebuffer must be given to the *new* method as *&mut [u8; BUFFER_SIZE]* (with BUFFER_SIZE = 320 * 240 * 2). This must be allocated by the user so that e.g. DMA-capable memory can be used (which speeds up SPI transfers significantly).

This is obviously platform-dependent; e.g. on ESP32 using Rust bindings for ESP-IDF it can be done like that:

```

let framebuf: &mut [u8; BUFFER_SIZE] = unsafe { &mut *(heap_caps_malloc(BUFFER_SIZE, MALLOC_CAP_DMA) as *mut [u8; BUFFER_SIZE]) };

```

The original README from ili9341-rs:

# `ili9341`

> A platform agnostic driver to interface with the ILI9341 (and ILI9340C) TFT
> LCD display

## What works

- Putting pixels on the screen
- Change the screen orientation
- Compatible with [embedded-graphics](https://docs.rs/embedded-graphics)

## TODO

- [ ] Expose more configuration options
- [ ] Scrolling
- [ ] Read video memory
- [ ] DMA API
- ???

## Examples

SOON

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the
work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
