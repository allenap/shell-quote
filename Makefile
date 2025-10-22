.PHONY: test
test:
	python3 test.py | bash -xe

.PHONY: check
check:
	cargo check --all-targets --all-features
	cargo check --all-targets --all-features --target aarch64-pc-windows-msvc
