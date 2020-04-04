# fresh

A simple sliding tile game. You can replace the image used by swapping out `assets/background.jpg` for whatever
Amethyst can handle (eg. `.png`s). Additionally, you can change the number of tiles in the board by changing the value
in `SimpleState::on_start` to something other than 4.

Todo items: 
* Make it not just crash when you solve it
* Make it so that the board size doesn't depend on the window size
* Scramble the board better when it's generated

## How to run

To run the game, use

```
cargo run --features "vulkan"
```

on Windows and Linux, and

```
cargo run --features "metal"
```

on macOS.