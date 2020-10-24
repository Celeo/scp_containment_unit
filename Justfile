default: build_release

build_release:
  @cargo build --release
  @cp target/release/scp_containment_unit .
  @strip ./scp_containment_unit
