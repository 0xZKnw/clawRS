@echo off
call "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat"
cd /d "C:\Users\crist\Desktop\LocaLM"
cargo build --release --features cuda
echo Build finished with exit code: %ERRORLEVEL%
pause
