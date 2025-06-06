.PHONY: build
build:
	RUSTFLAGS="--emit=llvm-bc" cargo build -p rts --release

	cp target/release/deps/rts.bc target/rts.bc

	opt \
		--internalize-public-api-list="noop,new_app,new_partial,apply_partial,copy,free_args,free_term,todo" \
		--passes="internalize,globaldce" \
		target/rts.bc \
		-o target/rts.bc

	opt \
		--passes="internalize" \
		target/rts.bc \
		-o target/rts.bc
