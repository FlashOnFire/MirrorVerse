# ✨ MirrorVerse ✨

Light ray reflection simulation in Rust, rendered with OpenGL.

All these simulations are done using Linear Algebra so it works for n-dimensional space.

This project is built using Rust for all the simulation parts. We heavily utilize the nAlgebra library for the calculations and also glium for the 3D rendering.

We used Flutter to make a GUI to run all the things.

The program is split into four main parts:

1. 📚 The library which really handles the simulations (mirror_verse).
2. 🏃‍♂️ The runner which takes a JSON, uses the simulator to generate the path, and runs a visualization of the simulation.
3. 🔀 The random generator which generates a random set of mirrors.
4. 🖥️ A graphical user interface to run all these tools easily.

## GUI

### 🛠️ Compilation

You should first build the Rust project into the Flutter assets:

```shell
# For Windows:
cargo build --release
copy target\release\generate_random_simulation_3d.exe mirror_verse_ui\assets
copy target\release\run_simulation_json_3d.exe mirror_verse_ui\assets

# For linux/macOS:
cargo build --release && \
cp target/release/generate_random_simulation_3d mirror_verse_ui/assets && \
cp target/release/run_simulation_json_3d mirror_verse_ui/assets
```

### 🚀 Run UI

```shell
cd mirror_verse_ui
flutter run --release
```

## CLI

### 🔬 Simulating

```shell
cargo run --release --bin run_simulation_json_3d -- "<path/to/simulation.json>"
```

### 🔄 Generating random mirror set

```shell
cargo run --release --bin generate_random_simulation_3d -- "<path/to/output.json>"
```