# MirrorVerse

Light ray reflection simulation in Rust, rendered with OpenGL

## Installation

Run the following commands in your terminal

```shell
git clone https://github.com/FlashOnFire/MirrorVerse.git
cd MirrorVerse
cargo build --release
```

The compiled executable requires you to pass it a path to
a json file representing the simulation you wish to run
examples of which can be found in the `assets` directory

You can retrieve it in `target/release` or run it using the following command:

```shell
cargo run "<path/to/simulation.json>"
```
