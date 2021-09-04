![image](https://user-images.githubusercontent.com/692124/132104593-6400eff5-75e4-412d-8499-b5dabd7d81f5.png)

Put your head on the desktop

![screencast](https://user-images.githubusercontent.com/692124/132104445-1d946673-0c8f-47cb-93c4-2577e76342c0.gif)

## Install

_A release is coming soon_

## Build

### Linux

Should work without hassle.

### Windows

OpenCV is a dependency and it requires build tools.

1. Install vcpkg
2. `vpgkg install llvm opencv4`
3. Copy `opencv_world4xx.dll` to `.\target\debug`

If you already have llvm, just add LLVM binaries to your PATH. It takes much time to install it with vcpkg.
