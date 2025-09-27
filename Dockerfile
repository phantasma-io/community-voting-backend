FROM rust:latest as builder

WORKDIR /build
COPY . .
RUN cargo build --release

FROM rust:latest AS runner
WORKDIR /

COPY --from=builder /build/target/release/community-voting-backend .
RUN /bin/sh -c set -ex; 	apt-get update; 	apt-get install -y --no-install-recommends 	libssl-dev; 	rm -rf /var/lib/apt/lists/* # buildkit

CMD ["./community-voting-backend"]
