BIN = ./target/debug/mirlvm
TEST = ./test
BUILD = build
OPTION = null
SSAFILE = null

$(BUILD): 
	cargo build

alltests: $(BUILD)
	@$(BIN) $(TEST)/branch_1.ssa > a.s
	gcc a.s

debug: $(BUILD)
	@$(BIN) $(OPTION) $(TEST)/$(SSAFILE)

clean:
	cargo clean