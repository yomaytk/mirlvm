BIN = ./target/debug/mirlvm
TEST = ./test
BUILD = build
OPTION = null
SSAFILE = null

$(BUILD): 
	cargo build

alltests: $(BUILD)
	@$(BIN) $(TEST)/fib.ssa > a.s
	gcc a.s

debug: $(BUILD)
	@$(BIN) $(OPTION) $(TEST)/$(SSAFILE) > debug.s
	gcc debug.s

clean:
	cargo clean