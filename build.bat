@echo off
call "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat"
set LIBCLANG_PATH=C:\Program Files\LLVM\bin
set PATH=C:\Program Files\CMake\bin;%PATH%
cargo build %*
