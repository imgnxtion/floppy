# GNU Makefile using Guile/Scheme for a touch of Lisp.
# Requires: GNU make built with Guile support. On macOS, install via Homebrew: `brew install make guile` and run with `gmake`.

# Pretty printing via Guile
PRINT = $(guile (begin (define (p . xs) (for-each display xs) (newline)) (p $(1))))

# Feature flags
CLI_FEATURES := --no-default-features --features cli
GUI_FEATURES := --no-default-features --features gui

.PHONY: help
help:
	@$(call PRINT,"Targets:")
	@$(call PRINT,"  build-cli     - Build CLI (default features)")
	@$(call PRINT,"  run-cli ARGS= - Run CLI with ARGS (e.g. ARGS=--help)")
	@$(call PRINT,"  install-cli   - Install CLI to cargo bin directory")
	@$(call PRINT,"  build-gui     - Build GUI (feature: gui)")
	@$(call PRINT,"  run-gui       - Run GUI (feature: gui)")
	@$(call PRINT,"  clean         - Clean target directory")

.PHONY: build-cli
build-cli:
	cargo build --bin meta $(CLI_FEATURES)

.PHONY: run-cli
run-cli: build-cli
	cargo run --bin meta $(CLI_FEATURES) -- $(ARGS)

.PHONY: install-cli
install-cli:
	cargo install --path . --bin meta $(CLI_FEATURES)

.PHONY: build-gui
build-gui:
	cargo build --bin meta-gui $(GUI_FEATURES)

.PHONY: run-gui
run-gui: build-gui
	cargo run --bin meta-gui $(GUI_FEATURES)

.PHONY: clean
clean:
	cargo clean

