FROM rust:1.78

WORKDIR /usr/src/vpn
COPY . .

RUN cargo install --path .

CMD ["vpn"]
