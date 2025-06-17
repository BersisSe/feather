FROM rust:latest AS build
ARG framework
WORKDIR /app
COPY . .
RUN cd benches && \
    rustup target add x86_64-unknown-linux-musl && \
    cargo build --target x86_64-unknown-linux-musl --locked --release --package ${framework}-bench

FROM scratch
ARG framework
COPY --from=build /app/benches/target/x86_64-unknown-linux-musl/release/${framework}-bench /app/server
EXPOSE 3000
CMD ["/app/server"]