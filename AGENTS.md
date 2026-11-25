# AGENTS.md

## Project Overview

**Nano Clock** (`nano-c-embassy`) is a bare-metal, `no_std` Rust firmware project that displays
messages on a Waveshare 7.5" e-paper display (7in5 v2) mounted on an nRF52840 board
(Cortex-M4F). It is built on the [Embassy](https://embassy.dev) async ecosystem.

- MCU: nRF52840 (1 MB flash, 256 KB RAM), SPI peripheral `TWISPI0` (SPI0) for the display
- Display: `epd-waveshare` `epd7in5_v2` driver, rendered with `embedded-graphics` (mono 6x10 font, white-on-black)
- Logging: `defmt` over RTT (`defmt-rtt`); panics are forwarded to the debugger via `panic-probe`
- The e-paper refresh is a **slow, full-frame, synchronous** operation (seconds) — a key constraint for any new display work

## Repository Layout

```
src/
  main.rs      # no_std entry: embassy_nrf::init, GPIO + SPI setup, E-Paper init, task spawning
  display.rs   # display_task: waits for DISPLAY_UPDATE, renders state.message, pushes frame to E-Paper
  message.rs   # publish_message_task: periodically writes state.message and signals DISPLAY_UPDATE
  state.rs     # shared state: STATE (Mutex) + DISPLAY_UPDATE (Signal)
build.rs       # copies memory.x to OUT_DIR, passes -Tlink.x/-Tdefmt.x and --nmagic to the linker
memory.x       # linker memory layout (see note below)
.cargo/config.toml  # default build target + DEFMT_LOG=trace
rust-toolchain.toml # Rust 1.100 pinned, thumbv7em-none-eabihf target
```

There are no unit tests, integration tests, or test infrastructure (bare-metal `no_std` binary).

## Build

The build target (`thumbv7em-none-eabihf`) and `DEFMT_LOG=trace` are set in `.cargo/config.toml`,
so plain cargo commands work from the repo root. The toolchain is pinned to Rust 1.100 by
`rust-toolchain.toml` (components: rust-src, rustfmt, llvm-tools, rust-analyzer — no clippy).

```sh
cargo build             # debug build → target/thumbv7em-none-eabihf/debug/nano-c-embassy
cargo build --release   # release build (profile.release has debug = 2)
cargo fmt               # formatting
```

No flashing tooling is configured in the repo (no probe-rs config); flashing the produced
binary to the board is done with an external tool of your choice.

### Linker / memory layout

`build.rs` places `memory.x` on the linker search path (re-runs when `memory.x` changes).
`memory.x` defines:

- `FLASH`: ORIGIN `0x00010000`, LENGTH `960K` — the first **64 KB of flash is reserved**
  (bootloader); firmware must not assume address `0x00000000`.
- `RAM`: ORIGIN `0x20000000`, LENGTH `256K`

## Architecture

Embassy tasks communicate through the globals in `src/state.rs`:

- `STATE: Mutex<ThreadModeRawMutex, State>` — holds the current `message: &str`
- `DISPLAY_UPDATE: Signal<ThreadModeRawMutex, ()>` — tells the display task to refresh

Flow:

1. `main` (`src/main.rs`) initializes the chip, binds the `TWISPI0` interrupt, wires the
   display pins, does an initial "hello world" frame, then spawns the two tasks.
2. `publish_message_task` (`src/message.rs`) is the content producer: every 10 s it locks
   `STATE`, updates `message`, releases, then calls `DISPLAY_UPDATE.signal(())`.
   (The intended long-term content source is the clock/time — see README.)
3. `display_task` (`src/display.rs`) loops: awaits `DISPLAY_UPDATE`, briefly locks `STATE`
   to copy out the message, renders it into a `Display7in5` framebuffer via the shared
   `render_message` helper (also used for the boot frame in `main`), calls the **blocking**
   `update_and_display_frame`
   Note: completing the `main` task is fine — the thread-mode executor keeps running the
   spawned tasks.

Display pin map (documented in `main.rs`): BUSY `P0.23`, CS `P1.02`, DC `P0.27`, RST `P0.21`,
PWR `P1.14` (driven high). SPI is TX-only at 4 MHz, mode 0, MSB first.

## Conventions & Gotchas

- **Task pattern**: tasks are `#[embassy_executor::task]` async fns spawned with
  `Spawner::spawn` (embassy-executor 0.10 style, executor-thread + executor-interrupt features).
  Use the same pattern for new tasks.
- **Shared state**: use `embassy_sync` with `ThreadModeRawMutex` as the mutex/signal raw
  implementation, mirroring `state.rs`.
- **Mutex discipline in the display path**: lock `STATE` only long enough to copy data out;
  never hold the lock across the seconds-long E-Paper `update_and_display_frame` call.
- **E-Paper is full-frame and slow**: every update rewrites the whole display and takes
  seconds. Batch changes; do not update on every tick.
- **`DISPLAY_UPDATE` is a lossy `Signal`**: a signal sent while the display task is mid-refresh
  (or in the tiny window between refresh and re-arming the wait) is dropped, and that update
  then waits up to one publish cycle (10 s). Fine at the current cadence; if updates must
  never be skipped, switch to a `Channel` or a dirty-flag re-check after each refresh.
- **embedded-graphics 0.8 API** is used: `Color::White`/`Color::Black`, `MonoTextStyleBuilder`,
  `Text::with_text_style`, `TextStyleBuilder` with `Baseline::Top`.
- **Timing**: embassy-time runs at 32 768 Hz tick (`tick-hz-32_768`), driven by RTC1
  (`time-driver-rtc1`).
- Keep the repo's no_std/no_main shape: no `std`, no `println!`. There's no debug probe so logging
  is pointless - write to the screen if needed.
