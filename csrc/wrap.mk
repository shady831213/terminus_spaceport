RUST_TARGET=../target/debug/libdpi_memory.a
RUST_DEP=../target/debug/libdpi_memory.d
TARGET=libdpi_memory.c.so
SRC=$(wildcard *.c)
OBJ=$(patsubst %.c,%.o,$(SRC))

CFLAG=-Wall -fPIC -I . -g
LFLAG= -Wl,-Bdynamic -lpthread -ldl -lm

all:$(TARGET)

$(TARGET):$(OBJ) $(RUST_TARGET)
	gcc -shared  $+ $(LFLAG) -o $@

$(OBJ):$(SRC)
	gcc -c $(CFLAG) $< -o $@

$(RUST_TARGET):$(RUST_DEP)
	cargo build

.PHONY:clean
clean:
	@rm -f $(OBJ) $(TARGET)