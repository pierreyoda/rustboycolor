FROM rust:1.63-slim

RUN apt-get -q update && apt-get -q install -y \
    libsdl2-dev \
    libsdl2-ttf-dev \
    && rm -r /var/lib/apt/lists/*

WORKDIR /usr/src/rustboycolor/

COPY . .

RUN cargo build
