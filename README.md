# CHIP-8 Emulator in Rust

![Brix](https://raw.githubusercontent.com/shiver/chip8/master/images/BRIX.gif)

A short weekend project to get some understanding around emulators and a little more experience with Rust.
If you have any suggestions or comments regarding either the code, emulators or Rust in general I would be happy to hear from you!

I have tested this on both Windows 10 and Arch Linux.

## Requirements

- CHIP-8 programs
    
    See `Resources used during development` section below.

- Rust 1.26+
- SDL2 development libraries

    ### Linux

    If you're running Linux, simply install the relevant package for your distribution. Such as `libsdl2-dev` for Ubuntu.

    ### Windows

    I have included the SDL2-2.0.8 pre-compiled binaries for `MSVC` and `MINGW`. However, I can only confirm having tested with the `MSVC` binaries.

## How to run

    $ cargo run -- <PROGRAM>

## Testing

    $ cargo test
    running 24 tests
    test test_add_const ... ok
    test test_add ... ok
    test test_assign_value ... ok
    ...

## Contributions

Contributions are welcome! Whether in the form of pull requests, suggestions, or comments. I would be happy to discuss any aspect of the project.

## Resources used during development

**CHIP-8 Info**:

- https://en.wikipedia.org/wiki/CHIP-8
- http://devernay.free.fr/hacks/chip8/C8TECH10.HTM

**Programs**:

- https://www.zophar.net/pdroms/chip8.htmll

## License

`chip8` is distributed under the terms of the MIT license.

See LICENSE.md for details.
