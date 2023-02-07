set positional-arguments

build:
    cargo build

run *args='':
    cargo build && sudo ./target/debug/rserial $@

connect:
    cargo build && sudo ./target/debug/rserial i /dev/ttyS0

simulate:
    sudo socat PTY,link=/dev/ttyS0,raw,echo=0 -

simulate-list:
    sudo lsof /dev/ttyS0

clean:
    cargo clean
