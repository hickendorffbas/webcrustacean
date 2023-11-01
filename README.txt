



# Installation



## Install SDL on mac:

brew install sdl2
(optionally) brew link sdl2
brew install sdl2_mixer
(optionally) brew link sdl2_mixer
brew install sdl2_image
(optionally) brew link sdl2_image
brew install sdl2_ttf
(optionally) brew link sdl2_ttf
brew install sdl2_gfx
(optionally) brew link sdl2_gfx


## If linking with SDL2 fails, run:

export LIBRARY_PATH="$LIBRARY_PATH:$(brew --prefix)/lib"




# Profiling (on Linux)


TODO: test and describe how to export debug symbols in release mode


perf record --call-graph=dwarf ./target/debug/bbrowser
perf report
