SET PATH=%PATH%;D:\vcpkg\installed\x64-windows\bin
SET INCLUDE=%INCLUDE%;D:\vcpkg\installed\x64-windows\include
SET LIB=%LIB%;D:\vcpkg\installed\x64-windows\lib

:: extremely ugly, how can we just pass the bin path to the exe somehow?
:: maybe add to path? but that is what I'm doing?
copy D:\vcpkg\installed\x64-windows\bin\*.dll .

cargo run
:: .\target\debug\webcrustacean.exe


:: possibly make a script that copies all dll's to a folder, and runs the exe there? but does that even work?


