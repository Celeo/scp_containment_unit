default: build

build: check
  @cargo build

check:
  @cargo check
  @cargo +nightly clippy

build_release: build
  @cargo build --release
  @cp target/release/scp_containment_unit .
  @strip ./scp_containment_unit

test:
  @cargo test
