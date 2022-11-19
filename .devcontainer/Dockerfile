ARG VARIANT="bullseye"
FROM mcr.microsoft.com/devcontainers/rust:1-${VARIANT}

RUN apt-get update && apt-get install -y libudev-dev socat screen
RUN cargo install just
