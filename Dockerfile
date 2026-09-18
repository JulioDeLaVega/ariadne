# ---- Build stage ----
FROM messense/rust-musl-cross:x86_64-musl AS builder
WORKDIR /app
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl

# ---- Runtime stage ----
FROM scratch AS runtime
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/ariadne /ariadne
EXPOSE 8080
CMD ["/ariadne"]