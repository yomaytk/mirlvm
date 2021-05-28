BIN = ./target/debug/mirlvm
TEST = ./test
BUILD = build
OPTION = null

$(BUILD): 
	cargo build

alltests: $(BUILD)
	@$(BIN) $(OPTION) $(TEST)/add.ssa

clean:
	cargo clean