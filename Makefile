.PHONY: rts
rts:
	RUSTFLAGS="--emit=llvm-bc" cargo build -p rts --release
