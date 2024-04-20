FROM rust:bookworm as build

ARG user=dsaw
RUN useradd -m $user
USER $user
WORKDIR /home/$user

COPY --chown=$user . .
RUN cargo install --path .

FROM debian:bookworm-slim as final

RUN apt-get update && \
    apt-get install -y msmtp msmtp-mta mailutils openssl ca-certificates && \
    rm -rf /var/lib/apt/lists/*

ARG user=dsaw
RUN useradd -m $user
USER $user
WORKDIR /home/$user

COPY --from=build /usr/local/cargo/bin/dedicated-server-availability-watcher /usr/local/bin/dedicated-server-availability-watcher

ENTRYPOINT ["dedicated-server-availability-watcher"]
