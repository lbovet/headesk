![image](https://user-images.githubusercontent.com/692124/132104593-6400eff5-75e4-412d-8499-b5dabd7d81f5.png)

![screencast](https://user-images.githubusercontent.com/692124/132104445-1d946673-0c8f-47cb-93c4-2577e76342c0.gif)

## Features

Headesk displays your webcam video on the desktop. Ideal for remote presentations and demos.

### Usage

* **Move Window**: `Left Click` and drag.
* **Resize Window**: `Mousewheel`.
* **Zoom Content**: `Ctrl + Mousewheel`.
* **Move Content**: `Ctrl + Left Click` and drag.
* **Change Camera**: `Right Click`.

### Background Removal

* Use a virtual camera like [XSplit VCam](https://www.xsplit.com/vcam) or equivalent.
* Automatic chroma key detection with a green screen.

## Install

* Windows: [Download](https://github.com/lbovet/headesk/releases) and unzip.
* Linux: [Download](https://github.com/lbovet/headesk/releases)
  - Requires `libopencv`
  - _Please help me to provide .rpm and .deb builds_.
* Mac OS: _Ain't no mac, please gimme one to build and test_.

## Build
_Instructions for developers_

### Linux

1. `cargo build`

### Windows

OpenCV is a dependency and it requires build tools.

1. Install vcpkg
2. `vpgkg install llvm opencv4`
3. Copy `opencv_world4xx.dll` to `.\target\debug`
4. `cargo build`

If you already have llvm, just add LLVM binaries to your PATH. It takes much time to install it with vcpkg.

### Mac OS

Please try and tell me if it works (I have no Mac).
