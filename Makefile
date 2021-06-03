BIN = ./target/debug/mirlvm
TEST = ./test
BUILD = build
OPTION = null

$(BUILD): 
	cargo build

alltests: $(BUILD)
	@$(BIN) $(OPTION) $(TEST)/call_1.ssa > a.s
	gcc a.s

debug: $(BUILD)
	@$(BIN) $(OPTION) $(TEST)/call_1.ssa

clean:
	cargo clean