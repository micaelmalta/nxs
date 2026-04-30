CC      = cc
CFLAGS  = -std=c99 -O2 -Wall -Wextra
LDFLAGS = -lm

.PHONY: all clean test bench

all: test bench

test: test.c nxs.c nxs.h
	$(CC) $(CFLAGS) -o test test.c nxs.c $(LDFLAGS)

bench: bench.c nxs.c nxs.h
	$(CC) $(CFLAGS) -o bench bench.c nxs.c $(LDFLAGS)

clean:
	rm -f test bench
