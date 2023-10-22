FROM debian:bookworm-20231009 AS build

ENV DEBIAN_FRONTEND="noninteractive"

RUN apt update
RUN apt -qq install -y --no-install-recommends curl ca-certificates build-essential pkg-config openssl libssl-dev libsqlite3-dev
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -q -y --default-toolchain 1.73.0

ENV PATH "${PATH}:/root/.cargo/bin"

WORKDIR /project
COPY Cargo.toml /project/

# Workaround to cache dependency libraries
RUN \
    mkdir src/ && \
    echo 'fn main() {}' > src/main.rs && \
    cargo build --release && \
    rm -Rvf src/

# Build actual code
COPY src/ /project/src/
RUN \
    touch src/main.rs && \
    cargo build --release

FROM debian:bookworm-20231009 AS run

ENV DEBIAN_FRONTEND="noninteractive"

# cron is required for whishlist update
# sqlite3 is required to interact with DB (instead we have an error with libsqlite3.so.0)
# openssl is required for whishlist (instead we have an error with libssl.so.3)
# ca-certificates is required for connect to a site through TLS (instead we have an
# "Runtime error: Failed to get <site>"
RUN apt -qq update && apt -qq install -y --no-install-recommends cron sqlite3 openssl curl ca-certificates

COPY entrypoint.sh /opt/entrypoint.sh
RUN chmod 0544 /opt/entrypoint.sh

COPY --from=build /project/target/release/wishlist /usr/local/bin/wishlist
COPY etc/wishlist.toml /etc/wishlist.toml
RUN mkdir -p /var/lib/wishlist/
RUN /usr/local/bin/wishlist add eva-ua pr229981-78182

EXPOSE 3585

RUN mkdir -p /etc/crontab.d/
RUN echo "*/10 *    * * *   root    /usr/local/bin/wishlist update" > /etc/crontab.d/wishlist
RUN chmod 0444 /etc/crontab.d/wishlist
RUN crontab /etc/crontab.d/wishlist

ENTRYPOINT /opt/entrypoint.sh
