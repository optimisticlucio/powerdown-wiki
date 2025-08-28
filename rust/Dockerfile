FROM rust:1.67

WORKDIR /usr/src/myapp
COPY . .

RUN cargo install --path .

CMD ["powerdown-wiki"]
# TODO: SET UP CARGO CHEF ONCE I GOT THIS THING ROUGHLY SET UP