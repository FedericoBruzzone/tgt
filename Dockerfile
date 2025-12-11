# use Trixie as the base image
FROM rust:1.91-trixie AS builder

# create workspace
RUN mkdir -p /app
WORKDIR /app
COPY . .

# build the application
RUN cargo build --release --features=download-tdlib

#
# end of file
#
