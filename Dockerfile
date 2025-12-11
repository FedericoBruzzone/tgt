# use Trixie as the base image
FROM rust:1.91-trixie AS builder

# create workspace
RUN mkdir -p /app
WORKDIR /app
COPY . .

# build dependencies

# build TDLib from source

RUN mkdir -p /deps/tdlib
RUN apt-get update && \
    apt-get install -y git cmake zlib1g-dev libssl-dev clang libc++-dev libc++abi-dev && \
    rm -rf /var/lib/apt/lists/*
RUN cd /deps/tdlib && \
    git clone https://github.com/tdlib/td.git . && \
    git checkout v1.8.0 && \
    mkdir build && cd build && \
    CXXFLAGS="-stdlib=libc++" CC=/usr/bin/clang CXX=/usr/bin/clang++ && \
    cmake -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX:PATH=../tdlib-install-dir .. && \
    cmake --build . --target install && \
    export LOCAL_TDLIB_PATH=/deps/tdlib/tdlib-install-dir

# build the application
RUN cargo build --release --features=local-tdlib

# final image
FROM debian:trixie-slim AS runtime

WORKDIR /app

COPY --from=builder /app/target/release/tgt /app/

RUN mkdir ~/.tgt -p

COPY --from=builder /app/config ~/.tgt/config

CMD [ "bash" ]

#
# end of file
#
