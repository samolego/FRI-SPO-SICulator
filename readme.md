# SIC/XE TUI simulator

This is a [SIC/XE](https://doi.org/10.1002/cae.21585) simulator written in Rust programming language.
It supports loading the object files and running the programs.

![Screenshot_20250213_165049](https://github.com/user-attachments/assets/e115e3a2-932e-4674-b835-565de5d364d8)


## Running

Make sure you have Rust installed. Then, run the following command:
```bash
cargo run --release
```

You can load the object files via "Load file" menu and navigate with `j` and `k` keys.

Note: "Text display" is currently unused.

## How to build

```bash
cargo build --release
```
