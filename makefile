DLL=organ.dll
DEBUG="target/i686-pc-windows-gnu/debug/$(DLL)"
RELEASE="target/i686-pc-windows-gnu/release/$(DLL)"

.PHONY: build release run run-release copy clean

build:
	cargo build --lib --target i686-pc-windows-gnu
release:
	cargo build --release --lib --target i686-pc-windows-gnu

run:
	wine savihost.exe target/i686-pc-windows-gnu/debug/$(DLL)
run-release:
	wine savihost.exe target/i686-pc-windows-gnu/release/$(DLL)

copy:
	cp -i $(DEBUG) ~/Sonido/Vsts/$(DLL)

copy-release:
	cp -i $(RELEASE) ~/Sonido/Vsts/$(DLL)

clean:
	cargo clean
