# VitaTTY

A Terminal Emulator for the PS Vita.

Currently just renders a triangle, but that's usually the hardest part.

## Building

Perform steps required to get [cargo vita](https://github.com/vita-rust/cargo-vita) working, and ensure you install the `vitacompanion` and `PrincessLog` modules on your vita.

do `rustup override set nightly` on this folder,

if you get an error involving `utimesnat`, you may need to edit your `$HOME/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/rustlib/src
/rust/library/std/src/sys/fs/unix.rs` to add `, target_os = "vita"` next to `target_os = "nuttx"` in the `set_times_impl` function

then to install (where `$VITA_IP` is the ip address of your vita):

```bash
cargo vita build vpk
cargo vita upload --vita_ip $VITA_IP --source --source target/armv7-sony-vita-newlibeabihf/debug/vitatty.vpk
```

Then use VitaShell to navigate to where `vitatty.vpk` was transferred to, and install it and run it.

Now you can use:

```bash
cargo vita build eboot --update --run # perhaps with --release
```

to send the new build and run it on your vita

Also, make sure to have `cargo vita logs` running somewhere to watch for `println!`s
