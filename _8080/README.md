# Lemurs 8080

This package provides an emulator for the Intel 8080 microprocessor. It models only the chip itself and can be supported with any value that supports the published Harness trait.

The package supports compiling without the "std" feature to remove dependencies on the std crate. This will require supplying a replacement panic handler and allocator.

You can use the script below to build a static library that can be used in C++ projects, which will produce a folder called `lemurs_8080_cpp` in the current directory containing a header file and both debug and release versions of the library:

### Linux/Mac OS X/ other POSIX
```bash
#!/bin/bash
mkdir lemurs_8080_cpp; cd lemurs_8080_cpp; L8080=$(pwd); mkdir debug release
cargo new --vcs none build_8080_cpp
(cd build_8080_cpp; cargo add lemurs-8080; cargo vendor)
(
	cd build_8080_cpp/vendor/lemurs-8080; cp -R include "$L8080";
	cargo +nightly -Z unstable-options build --no-default-features --features "cpp $1" \
		--out-dir "${L8080}/debug"
	cargo +nightly -Z unstable-options build --no-default-features --features "cpp $1" \
		--out-dir "${L8080}/release" --release
)
rm -r build_8080_cpp
```

Lemurs is intended to be a collection of chip emulation packages. Currently only the i8080 is supported.
