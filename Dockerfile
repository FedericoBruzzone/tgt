# use Trixie as the base image
FROM rust:1.91-trixie AS builder

# create workspace
RUN mkdir -p /app
WORKDIR /app
COPY . .

# build the application
RUN cargo build --release --features=download-tdlib

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
