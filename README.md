# Webcrustacean

This is a very, very basic start of a browser. A lot is not implemented, but its is functional for viewing very basic webpages.

__Note:__ Unlike most new browsers, this is not a chromium wrapper, but an actual new implementation of rendering webpages. See the scope section for more info.



## Installation

Currently, no builds are provided. So the way to run webcrustacean is to install rust, and then run:

```cargo run```



## Scope

Although we use libaries for several things, we don't want to use a library for any core web technology. This means we want to build the following ourselves and not depend on libraries for it:

- parsing html, css and javascript
- compute the layout of the page
- the DOM
- all interactivity on the page


We do use libraries for the following functionality:

- gathering user input and rendering (SDL2)
- doing HTTP network requests (reqwest)
- loading different image formats (image)
- accessing the clipboard (arboard)



## Development Installation


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
