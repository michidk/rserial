set positional-arguments

run *args='':
    cargo build && sudo ./target/debug/rserial $@

simulate:
    sudo socat PTY,link=/dev/ttyS0,raw,echo=0 -

simulate-list:
    sudo lsof /dev/ttyS0
