# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]

### Added

- A `default_handler!` macro to override the default exception handler.

- An `exception!` macro to override the handler for a particular exception.

### Changed

- The FPU will now be enabled before `main` if the target has FPU support.

- [breaking-change] the features "panic-over-itm" and "panic-over-semihosting"
  has been removed. the `panic_fmt` language item is now *not* included by
  default. An opt-in feature named "abort-on-panic" can be enabled to make this
  crate provide a `panic_fmt` implementation that simply aborts.

- [breaking-change] The sections `.rodata.{exceptions,interrupts}` have been
  renamed to `.vector_table.{exceptions,interrupts}`. This break the old
  mechanism for registering exceptions (`static EXCEPTIONS`); use the new ones:
  `default_handler!` and `exception!`.

## [v0.2.4] - 2017-06-03

### Added

- A non-allocatable `.stlog` section to support the [`stlog`] logging framework.

[`stlog`]: https://crates.io/crates/stlog

## [v0.2.3] - 2017-05-30

### Added

- A `_stext` symbol which can be specified in the linker script to customize the
  location of the `.text` section. If not specified the `.text` section will be
  placed right after the `.vector_table` section.

## [v0.2.2] - 2017-05-27

### Added

- A `_sheap` symbol where the heap can be located.

### Changed

- The linker sections have renamed / reorder to make `arm-none-eabi-size -A`
  more useful. You'll now see something like this:

```
$ arm-none-eabi-size -A hello
hello  :
section                size        addr
.vector_table          1024   134217728
.text                   288   134218752
.rodata                  14   134219040
```

- `cortex-m-rt::reset_handler` is now the entry point of all programs that link
  to `cortex-m-rt`. This makes GDB's `load` command work correctly. It will now
  set the Program Counter to `reset_handler` after flashing the program so
  there's no need to reset the microcontroller after flashing.

- Renamed `__exceptions` and `__interrupts` symbols, which are only used
  internally, to `_eexceptions` and `_einterrupts` respectively for consistency.

### Fixed

- Include input `.text` and `.rodata` sections (note: no suffix as in
  `.text.foo`) in the output file. (C) Code compiled without the equivalent
  `-ffunction-sections` / `-fdata-sections` may place stuff in those unsuffixed
  sections.

## [v0.2.1] - 2017-05-07

### Fixed

- Do not load the `.debug_gdb_script` section in flash. It's only needed for
  debugging.

## [v0.2.0] - 2017-04-27

### Changed

- [breaking-change] the `_stack_start` symbol is now required and must be
  provided in the `memory.x` file when using the "linker-script" feature. This
  symbol indicates where in memory the call stack will be allocated.

## [v0.1.3] - 2017-04-25

### Fixed

- A `rustdoc` warning

## [v0.1.2] - 2017-04-22

### Changed

- Unclutter the `reset_handler` function for a better debugging experience.

## [v0.1.1] - 2017-04-15

### Changed

- Improved linker error messages

## v0.1.0 - 2017-04-12

Initial release

[Unreleased]: https://github.com/japaric/cortex-m-rt/compare/v0.2.4...HEAD
[v0.2.4]: https://github.com/japaric/cortex-m-rt/compare/v0.2.3...v0.2.4
[v0.2.3]: https://github.com/japaric/cortex-m-rt/compare/v0.2.2...v0.2.3
[v0.2.2]: https://github.com/japaric/cortex-m-rt/compare/v0.2.1...v0.2.2
[v0.2.1]: https://github.com/japaric/cortex-m-rt/compare/v0.2.0...v0.2.1
[v0.2.0]: https://github.com/japaric/cortex-m-rt/compare/v0.1.3...v0.2.0
[v0.1.3]: https://github.com/japaric/cortex-m-rt/compare/v0.1.2...v0.1.3
[v0.1.2]: https://github.com/japaric/cortex-m-rt/compare/v0.1.1...v0.1.2
[v0.1.1]: https://github.com/japaric/cortex-m-rt/compare/v0.1.0...v0.1.1
