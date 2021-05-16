BIN = ./target/debug/mirlvm
TEST = ./test
BUILD = build

$(BUILD): 
	cargo build

alltests: $(BUILD)
	$(BIN) $(TEST)/ret_word.ssa

clean:
	cargo clean