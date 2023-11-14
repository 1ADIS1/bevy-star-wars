# This is a template for my Rust projects, used to be build with VSCode on Windows.

* **Bevy 0.11**
* **Parry2D**
* **Rand**

_.vscode_ contains *tasks.json* that is used for executing rust code with shortcut: <code>"CTRL + SHIFT + B"</code>.

_.cargo_ contains code for dynamic lynking and faster compiling.

_rust-toolchain.toml_ is used for turning on the rust nightly compiler.

# WASM Build

**Run the following to build for web:** 

1. Install or update _**wasm-bindgen**_ , _**wasm-bindgen-cli**_ , **_wasm-opt_** dependencies: <code>cargo update -p wasm-bindgen</code>, <code>cargo update -p wasm-bindgen-cli</code>
2. Comment dynamic linking and optimisations in **cargo.toml** and execute <code>cargo build --release --target wasm32-unknown-unknown</code>.
3. Optimise the wasm build <code>wasm-opt -Oz out/bevy_project_template_bg.wasm --output out/bevy_project_template_bg.wasm</code>
