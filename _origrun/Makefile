TARGET	= _origrun.exe

ifeq ($(DEBUG), 1)
	MODE = debug
	RFLAGS =
else
	MODE = release
	RFLAGS = --release
endif

# ------------------------------------------------

all: $(TARGET)

$(TARGET): target\$(MODE)\$(TARGET)
	cp -f $< $@
	
target\$(MODE)\$(TARGET):
	cargo build $(RFLAGS)
	
clean:
	rm -f $(TARGET)

# ------------------------------------------------

.PHONY: clean
