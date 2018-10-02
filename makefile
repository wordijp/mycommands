BASE_EXE	= _origrun.exe

# ---------------------------

BATS	= \
	php-linter.bat

BATS_INTERNAL	= \
	internal\php-lint-run.bat \
	internal\phpmd-run.bat \
	internal\vim-arg.bat \
	internal\xexec.bat

EXES			= $(patsubst %, %, $(BATS:.bat=.exe))
EXES_INTERNAL	= $(patsubst %, %, $(BATS_INTERNAL:.bat=.exe))

# -----------------------------------------------

all:
	make -C _origrun

# ---

release: $(BASE_EXE) mklink
	
$(BASE_EXE): _origrun/$(BASE_EXE)
	cp -f $< $@

mklink: $(BATS) $(BATS_INTERNAL) $(EXES) $(EXES_INTERNAL) internal\_preproc.rb
	
$(BATS):
	@cmd /c "mklink $@ _origscript.bat"
	
$(BATS_INTERNAL):
	@cmd /c "mklink $@ ..\_origscript.bat"

$(EXES):
	@cmd /c "mklink $@ _origrun.exe"
	
$(EXES_INTERNAL):
	@cmd /c "mklink $@ ..\_origrun.exe"

internal\_preproc.rb:
	@cmd /c "mklink $@ ..\_preproc.rb"

# ---

clean:
	make clean -C _origrun
	rm -f $(BATS)
	rm -f $(BATS_INTERNAL)
	rm -f $(BASE_EXE)
	rm -f $(EXES)
	rm -f $(EXES_INTERNAL)
	rm -f internal\_preproc.rb

# -----------------------------------------------

.PHONY: clean
