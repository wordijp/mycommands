TARGET	= _origrun.exe

LDFLAGS	= -w -s

SRCS	= main.go


# ------------------------------------------------

all: $(TARGET)

$(TARGET): $(SRCS)
	go build -ldflags "$(LDFLAGS)" -o $@
	
clean:
	rm -f $(TARGET)

# ------------------------------------------------

.PHONY: clean
