SRCS = \
	php-linter.bat

INTERNAL_SRCS = \
	internal\php-lint-run.bat \
	internal\phpmd-run.bat \
	internal\vim-arg.bat \
	internal\xexec.bat

# -----------------------------------------------

all: $(SRCS) $(INTERNAL_SRCS)
	
$(SRCS):
	@cmd /c "mklink $@ _orig.bat"
	
$(INTERNAL_SRCS):
	@cmd /c "mklink $@ ..\_orig.bat"
	
clean:
	rm -f $(SRCS)
	rm -f $(INTERNAL_SRCS)

# -----------------------------------------------

.PHONY: clean
