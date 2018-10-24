TARGET	= _origrun.exe

SRCS	= src/main.rs

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
	
target\$(MODE)\$(TARGET): $(SRCS)
	cargo build $(RFLAGS)
	
clean:
	rm -f $(TARGET)

# ------------------------------------------------

.PHONY: clean
