# use Trixie as the base image
FROM rust:1.91-trixie AS builder

# create workspace
RUN mkdir -p /app
WORKDIR /app
COPY . .

# build dependencies

# build TDLib from source

RUN mkdir -p /deps/tdlib

RUN apt update && \
    apt install -y make git zlib1g-dev libssl-dev gperf cmake clang libc++-dev libc++abi-dev \
        pkg-config libasound2-dev && \
    rm -rf /var/lib/apt/lists/*

# build TDLib with v1.8.0 version as base and using clang compiler
RUN cd /deps/tdlib && \
    git clone https://github.com/tdlib/td.git && \
    cd td && \
    git checkout 6d509061574d684117f74133056aa43df89022fc && \
    export CXXFLAGS="-stdlib=libc++" && export CC=/usr/bin/clang && export CXX=/usr/bin/clang++ && \
    cmake -S . -B build -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=/deps/tdlib/tdlib-install-dir && \
    cmake --build build --target install -j$(nproc)

ENV LOCAL_TDLIB_PATH=/deps/tdlib/tdlib-install-dir

# build the application
RUN cargo build --release --features=default

# final image
FROM debian:trixie-slim AS runtime

WORKDIR /app

COPY --from=builder /app/target/release/tgt /usr/bin/
COPY --from=builder /deps/tdlib/tdlib-install-dir/lib/libtdjson.so* /usr/lib/

RUN mkdir -p /root/.config/tgt /root/.local/share/tgt

COPY --from=builder /app/config /root/.config/tgt/config

RUN apt update && \
    apt install -y libc++1 libasound2 && \
    rm -rf /var/lib/apt/lists/*

# opening bash shell to run tgt interactively
CMD [ "bash" ]

#
# end of file
#
