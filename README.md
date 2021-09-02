# headesk

Put your head on the desktop

## Build

### Linux

Should work without hassle.

### Windows

OpenCV is a dependency and it requires build tools.

1. Install vcpkg
2. `vpgkg install llvm opencv4`
3. Copy `opencv_world4xx.dll` to `.\target\debug`

If you already have llvm, just add LLVM binaries to your PATH. It takes much time to install it with vcpkg.
