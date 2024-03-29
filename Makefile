.PHONY: local
local:
	cargo build --release

.PHONY: run
run:
ifndef ARGS
	@echo Run "make run" with ARGS set to pass arguments…
endif
	cargo run --release -- $(ARGS)

.PHONY: build-linux
build-linux:
	cargo build --target x86_64-unknown-linux-musl --release --locked
	strip target/x86_64-unknown-linux-musl/release/anek
	upx --lzma target/x86_64-unknown-linux-musl/release/anek

.PHONY: build-win
build-win:
	RUSTFLAGS="-C linker=x86_64-w64-mingw32-gcc" cargo build --target x86_64-pc-windows-gnu --release --locked
	strip target/x86_64-pc-windows-gnu/release/anek.exe
	upx --lzma target/x86_64-pc-windows-gnu/release/anek.exe

.PHONY: build-apple
build-apple:
	cargo build --target x86_64-apple-darwin --release --locked
	strip target/x86_64-apple-darwin/release/anek
	upx --lzma target/x86_64-apple-darwin/release/anek
