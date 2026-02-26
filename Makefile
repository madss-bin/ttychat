PREFIX ?= /usr/local
BINDIR ?= $(PREFIX)/bin
TARGET = target/release/ttychat

.PHONY: all build install uninstall clean

all: build

build:
	cargo build --release

install:
	@if [ ! -f $(TARGET) ]; then \
		echo "Error: $(TARGET) not found. Build the project first with 'make build' or 'cargo build --release'."; \
		exit 1; \
	fi
	install -Dm755 $(TARGET) $(DESTDIR)$(BINDIR)/ttychat

uninstall:
	rm -f $(DESTDIR)$(BINDIR)/ttychat

clean:
	cargo clean
