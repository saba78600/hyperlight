CFLAGS=-std=c11 -g -fno-common -Wall -Wno-switch

# Release settings
VERSION ?= 0.1.0
CFLAGS_RELEASE=-std=c11 -O2 -fno-common -Wall -Wno-switch -s

SRCS=$(wildcard *.c)
OBJS=$(SRCS:.c=.o)

TEST_SRCS=$(wildcard test/*.c)
TESTS=$(TEST_SRCS:.c=.exe)

# Stage 1

hyperlight: $(OBJS)
	$(CC) $(CFLAGS) -o $@ $^ $(LDFLAGS)

.PHONY: release
release: hyperlight-release hyperlight-$(VERSION)-linux.tar.gz

hyperlight-release: $(OBJS)
	$(CC) $(CFLAGS_RELEASE) -o $@ $^ $(LDFLAGS)

hyperlight-$(VERSION)-linux.tar.gz: hyperlight-release README.md LICENSE
	@rm -f $@
	tar -czf $@ hyperlight-release README.md LICENSE

$(OBJS): hyperlight.h

test/%.exe: hyperlight test/%.c
	./hyperlight -Iinclude -Itest -c -o test/$*.o test/$*.c
	$(CC) -pthread -o $@ test/$*.o -xc test/common

test: $(TESTS)
	for i in $^; do echo $$i; ./$$i || exit 1; echo; done
	test/driver.sh ./hyperlight

test-all: test test-stage2

# Stage 2

stage2/hyperlight: $(OBJS:%=stage2/%)
	$(CC) $(CFLAGS) -o $@ $^ $(LDFLAGS)

stage2/%.o: hyperlight %.c
	mkdir -p stage2/test
	./hyperlight -c -o $(@D)/$*.o $*.c

stage2/test/%.exe: stage2/hyperlight test/%.c
	mkdir -p stage2/test
	./stage2/hyperlight -Iinclude -Itest -c -o stage2/test/$*.o test/$*.c
	$(CC) -pthread -o $@ stage2/test/$*.o -xc test/common

test-stage2: $(TESTS:test/%=stage2/test/%)
	for i in $^; do echo $$i; ./$$i || exit 1; echo; done
	test/driver.sh ./stage2/hyperlight

# Misc.

clean:
	rm -rf hyperlight tmp* $(TESTS) test/*.s test/*.exe stage2
	find * -type f '(' -name '*~' -o -name '*.o' ')' -exec rm {} ';'

.PHONY: test clean test-stage2
