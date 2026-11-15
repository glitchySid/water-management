# Water pump controller (ESP32, Rust std)

Controls a 1.5 HP municipal-water pump through a relay/contactor. The relay is
only the low-voltage control side; mains switching belongs to a properly rated
contactor.

## File tree

```
water-management/
├── .cargo/config.toml      # xtensa target, ldproxy, espflash runner
├── .gitignore
├── Cargo.toml
├── README.md
├── build.rs                # embuild env for esp-idf-sys
├── rust-toolchain.toml     # espup's "esp" toolchain
├── sdkconfig.defaults
├── shell.nix               # NixOS FHS build shell
└── src/
    ├── main.rs             # init + 200 ms loop
    ├── config.rs           # pins, timings, relay polarity, sim/real switch
    ├── motor.rs            # Motor: new/on/off/is_on
    ├── probe.rs            # Probe: water_present (5 samples, majority)
    ├── mode.rs             # Mode + debounced 3-position Switch
    └── pump.rs             # state, safety checks, cooldown, fault handling
```

`esp-idf-svc` is only used for `EspLogger`, which connects `log` to the ESP-IDF
serial logger. Everything else uses `esp-idf-hal`.

## Wiring (real mode)

| Signal | ESP32 pin | Connects to | Notes |
|---|---|---|---|
| Relay control | GPIO26 | Relay module IN | Active LOW. Relay contacts switch the contactor coil, never the pump directly. |
| Probe drive | GPIO25 | Probe electrode A | Driven HIGH only during a reading (about 105 ms). |
| Probe read | GPIO27 | Probe electrode B | Internal pull-down. Water bridges A/B and pulls it HIGH. |
| Switch ON | GPIO32 | 3-position switch throw 1 | Internal pull-up; switch shorts to GND. |
| Switch OFF | GPIO33 | 3-position switch throw 3 | Internal pull-up; switch shorts to GND. |
| Ground | GND | Switch common, relay GND, probe common | Common reference. |

Switch positions: throw 1 = `On`, throw 3 = `Off`, center (neither pin low) =
`Auto`. If both pins read low the code chooses `Off`. `Auto` runs the pump only
while the probe sees water and restarts it when water returns. `On` is a manual
override that starts the pump even if the probe is dry; a dry trip still clears
itself when water returns.

Suggested extras for the real build: 10 kΩ pull-up from GPIO26 to 3.3 V so the
relay stays off while the ESP32 is resetting/booting, a fuse/breaker and earth
on the mains side, and a contactor rated for the pump's locked-rotor current.
Keep the probe electrodes out of the potable-water path unless they are
food-safe / isolated; the pulsed drive limits corrosion but does not make the
probe potable-rated by itself.

## Simulation wiring (ESP32 only)

| Simulation signal | Wiring |
|---|---|
| Relay / motor | Onboard LED on GPIO2 (active HIGH). Nothing to wire. |
| Water present | Jumper GPIO25 ↔ GPIO27. Remove the jumper for dry. |
| Manual ON | Jumper GPIO32 ↔ GND, leave it connected. |
| Manual OFF | Jumper GPIO33 ↔ GND, leave it connected. |
| AUTO | No jumper on GPIO32/GPIO33. |

With the internal pull-ups, an unconnected GPIO32/33 means "not selected", so
the switch sits in `Auto`.

## Build and flash

Prerequisites (Linux): Rust via `rustup`, `git`, `cmake`, `ninja`, Python 3 with
venv, `pkg-config`, and libudev for espflash.

```sh
# once
cargo install espup espflash ldproxy
espup install
. $HOME/export-esp.sh          # every new shell

# build + flash (espflash runner is in .cargo/config.toml)
cargo run --release
cargo run --release --features simulation
```

Or flash explicitly (the feature is baked in at build time; the path is the same):

```sh
cargo build --release
cargo build --release --features simulation
espflash flash --monitor target/xtensa-esp32-espidf/release/water-management
```

To watch logs without flashing again: `espflash monitor`.

### NixOS notes

`shell.nix` puts the build tools in an FHS environment, so the glibc-linked
ESP-IDF and espup binaries run without `nix-ld`. First-time setup:

```sh
nix-shell
cargo +stable install espup   # once
espup install                 # once
exit && nix-shell             # shell.nix now sources ~/export-esp.sh
cargo build --release --features simulation
```

`espflash` and `ldproxy` come from nixpkgs. Add your user to the `dialout`
group if espflash cannot open `/dev/ttyUSB*`.

## Simulation test sequence

Open the monitor (`cargo run --release --features simulation`, or
`espflash monitor`) and follow the log.

1. **Boot / safe default.** No jumpers at all (Auto, dry). Expect
   `Motor OFF (boot)`, then `Mode: Auto`; the LED stays off.
2. **Auto starts with water.** Jumper GPIO25↔GPIO27 (wet). Expect `Motor ON`
   and the LED on, with no switch change.
3. **Dry trip.** Pull the GPIO25↔GPIO27 jumper while the LED is on. Within
   about 5 s expect `Fault (Dry): no water at probe` and
   `Motor OFF: no water at probe`; LED off.
4. **Auto restart.** Reconnect GPIO25↔GPIO27. After the 10 s cooldown expect
   `Water detected, dry fault cleared` and `Motor ON`. No mode change needed.
5. **Manual On.** Jumper GPIO32↔GND. Expect `Mode: On`; it runs even if the
   probe is dry (and then trips after 5 s).
6. **Dry trip in On.** Remove the water jumper while running in On. It stops,
   then restarts by itself when water is reconnected after the cooldown.
7. **Max run trip.** With water present and On, wait 60 s from start. Expect
   `Fault (MaxRun): max run time exceeded`; LED off. This fault does not clear
   itself: move to Off (`Fault cleared: MaxRun`) before it will run again.
8. **Cooldown.** After any stop, reconnecting water or moving to On does nothing
   for 10 s in simulation (5 min real). Watch `Motor ON` appear after it.
9. **Brownout/reboot check.** Reset the board: the LED must be off during boot
   and stay off unless the probe is wet in Auto.

Normal-mode values: water check at 10 s, max run 30 min, cooldown 5 min, relay
GPIO26 active LOW.

## What to build next (not implemented)

- An NTP schedule on top of the Auto probe check (start/stop windows, keeping
  the same safety checks).
- Tank-full / float switch input, with the pump stopped when the tank is full.
- Persist the fault cause across reboots, and optionally blink an LED code.
- A physical emergency stop that cuts the contactor coil, independent of the MCU.
