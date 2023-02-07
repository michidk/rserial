ARG VARIANT="bullseye"
FROM mcr.microsoft.com/devcontainers/rust:1-${VARIANT}

RUN apt-get update && apt-get install -y libudev-dev socat screen
RUN cargo install just

# hotfix because cargo fails otherwise for some reason
RUN chmod -R 777 /usr/local/cargo
