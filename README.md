# ✨ MirrorVerse ✨

Light ray reflection simulation with 3D rendering.

Built with [Rust](https://www.rust-lang.org/), using [nalgebra](https://nalgebra.org/) for the linear algebra, and [glium](https://github.com/glium/glium) for the graphical rendering.

GUI app built with [Flutter](https://flutter.dev/)

This project is split into four main parts:

1. 📚 The library which really handles the simulations (mirror_verse).
2. 🏃‍♂️ A runner which takes a JSON, generates ray's the path, and runs a visualization of the simulation.
3. 🔀 A random simulation generator which generates a random set of mirrors and rays.
4. 🖥️ A Flutter GUI app graphical for users who wish to run these tools without the terminal.

## GUI

### 🛠️ Compilation

Build the Rust project and move the emitted executables into the Flutter assets:

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

### 🚀 Running the UI

```shell
cd mirror_verse_ui
flutter run --release
```

## CLI

### 🔬 Running a simulation from a JSON file

```shell
cargo run --release -p run_sim_json "<path/to/simulation.json>"
```

### 🔄 Generating random simulation

```shell
cargo run --release -p gen_rand_sim "<path/to/output.json> <dimension> <num_rays> <num_mirrors>"
```
