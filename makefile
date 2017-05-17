DLL=organ.dll
DEBUG="target/i686-pc-windows-gnu/debug/$(DLL)"
RELEASE="target/i686-pc-windows-gnu/release/$(DLL)"

CARGO=wine /home/arnaud/.wine/drive_c/Program\ Files/Rust\ stable\ GNU\ 1.13/bin/cargo.exe

.PHONY: build release run run-release copy clean

# Es muy importante lo que está en build.rs

build:
	cargo build --lib --target i686-pc-windows-gnu
	cp target/i686-pc-windows-gnu/debug/$(DLL) ./
release:
	cargo build --release --lib --target i686-pc-windows-gnu
	cp target/i686-pc-windows-gnu/release/$(DLL) ./

run:
	wine savihost.exe $(DLL)

copy:
	cp -i $(DLL) ~/Sonido/Vsts/$(DLL)

clean:
	cargo clean
