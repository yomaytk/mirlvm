BIN = ./target/debug/mirlvm
TEST = ./test
BUILD = build
OPTION = null
OPTION2 = null
SSAFILE = null

$(BUILD): 
	cargo build

alltests: $(BUILD)
	@$(BIN) $(TEST)/fib.ssa > a.s
	gcc a.s

debug: $(BUILD)
	@if [ $(OPTION) = "null" -o $(OPTION) = "-O1" ]; then \
		$(BIN) $(TEST)/ret42.ssa > debug.s; \
		gcc debug.s; \
	else \
		$(BIN) $(OPTION) $(OPTION2) $(TEST)/$(SSAFILE) > out_debug.txt; \
		less out_debug.txt; \
	fi

clean:
	cargo clean