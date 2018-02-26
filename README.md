# rbreakout [![][img_license]](#license) [![][img_loc]][loc]

[img_license]: https://img.shields.io/badge/License-MIT_or_Apache_2.0-blue.svg
[img_loc]: https://tokei.rs/b1/github/AntonHakansson/rbreakout

A Breakout clone written in Rust

![rbreakout](https://user-images.githubusercontent.com/15860608/36357650-523b7d9a-1501-11e8-961c-b82835b1fe31.gif)

```
Usage:
    target/debug/rbreakout [OPTIONS]

A simple breakout clone built with rust, playable in the terminal

optional arguments:
  -h,--help             show this help message and exit
  -w,--width WIDTH      Preferable game width greater than 32, gets scaled
                        according to the in-game cell width
  -h,--height HEIGHT    Preferable game height greater than 20
  -f,--fill             Fill game to current terminal size
```
