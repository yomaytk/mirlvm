BIN = ./target/debug/mirlvm
TEST = ./test
BUILD = build

$(BUILD): 
	cargo build

alltests: $(BUILD)
	$(BIN) $(TEST)/add.ssa

clean:
	cargo clean