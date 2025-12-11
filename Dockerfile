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
    apt install -y make git zlib1g-dev libssl-dev gperf cmake clang libc++-dev libc++abi-dev && \
    rm -rf /var/lib/apt/lists/*

RUN cd /deps/tdlib && \
    git clone https://github.com/tdlib/td.git && \
    cd td && \
    git checkout v1.8.0 && \
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
COPY --from=builder /root/.tgt/tdlib/lib/libtdjson.so.1.8.29 /usr/lib/

RUN mkdir ~/.tgt -p

COPY --from=builder /app/config ~/.tgt/config

CMD [ "bash" ]

#
# end of file
#
