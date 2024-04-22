# Rustsweeper

![Minesweeper Demo](demo.gif)

## Overview

This repository contains a basic command-line Minesweeper program written in Rust.

## Installation

To play the Minesweeper game, you need to have Rust installed on your system. If Rust is not installed, you can download and install it from [here](https://www.rust-lang.org/tools/install).

Once Rust is installed, follow these steps:

1. Clone this repository to your local machine:

   ```
   git clone https://github.com/AidenPierce616/rustsweeper.git
   ```

2. Navigate into the project directory:

   ```
   cd rustsweeper
   ```

3. Build the project using Cargo:

   ```
   cargo build --release
   ```

4. Run the game:

   ```
   cargo run --release
   ```

## Usage

- **Mouse Controls (with GUI)**:
  - Click on a cell to reveal it.
  - Press `F` to flag a cell as a potential mine.
  - Press `Esc` at any time to return to the main menu.


- **Keyboard Controls (without GUI)**:
  - Use `W`, `A`, `S`, `D` keys to move the selected cell.
  - Press `C` to reveal the selected cell.
  - Press `F` to flag the selected cell as a potential mine.
  - Press `Esc` at any time to return to the main menu.

You can customize controls and adjust game difficulty using the in-game menu:
- Go to the main menu.
- Select the "Controls" option to change input preferences between mouse and keyboard.
- Select the "Difficulty" option to choose from predefined difficulty levels: Easy, Normal, and Hard.
- Alternatively, choose "Custom" to specify custom settings, including board width (max 40), board height (max 28), and the number of mines.

## Contributing

Contributions are welcome! If you find any bugs or have suggestions for improvement, feel free to open an issue or submit a pull request.
