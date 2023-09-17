# Lemurs 8080

This package provides an emulator for the Intel 8080 microprocessor. It models only the chip itself and can be supported with any value that supports the published Harness trait.

The package supports compiling without the "std" feature to remove dependencies on the std crate. This will require supplying a replacement panic handler and allocator.

Lemurs is intended to be a collection of chip emulation packages. Currently only the i8080 is supported.