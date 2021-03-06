BIN = ./target/debug/mirlvm
TEST = ./test
BUILD = build
OPTION = null
OPTION2 = null
OPTION3 = null
SSAFILE = null

$(BUILD): 
	cargo build

alltests: $(BUILD)
	@$(BIN) $(TEST)/fib.ssa > a.s
	gcc a.s

debug: $(BUILD)
	@if [ $(OPTION) = "null" ]; then \
		$(BIN) $(OPTION) $(OPTION2) $(OPTION3) $(TEST)/$(SSAFILE) > debug.s; \
		gcc -static debug.s; \
	else \
		$(BIN) $(OPTION) $(OPTION2) $(OPTION3) $(TEST)/$(SSAFILE) > out_debug.txt; \
		less out_debug.txt; \
	fi

clean:
	cargo clean