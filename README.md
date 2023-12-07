# Webcrustacean

This is a very, very basic start of a browser. A lot is not implemented, but its is functional for viewing very basic webpages.



## Installation


### Linux (ubuntu)

#### install SDL

```sudo apt-get install libsdl2-dev libsdl2-gfx-dev libsdl2-image-dev libsdl2-mixer-dev libsdl2-ttf-dev```



### Mac OS:

#### install SDL

```
brew install sdl2
(optionally) brew link sdl2
brew install sdl2_gfx
(optionally) brew link sdl2_gfx
brew install sdl2_image
(optionally) brew link sdl2_image
brew install sdl2_mixer
(optionally) brew link sdl2_mixer
brew install sdl2_ttf
(optionally) brew link sdl2_ttf
```


#### If linking with SDL2 fails, run:

```export LIBRARY_PATH="$LIBRARY_PATH:$(brew --prefix)/lib"```



### Windows

#### install SDL

see <https://github.com/Rust-SDL2/rust-sdl2#windows-msvc>



## Profiling (on Linux)

TODO: test and describe how to export debug symbols in release mode

perf record --call-graph=dwarf ./target/debug/webcrustacean
perf report
