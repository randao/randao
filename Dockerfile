# Compile golang
FROM ubuntu:20.04 as randao-builder
ENV PATH=/root/.cargo/bin:$PATH
RUN apt-get update \
  && DEBIAN_FRONTEND=noninteractive apt-get install libc-dev make git curl wget jq ssh python3-pip clang libclang-dev llvm-dev libleveldb-dev musl-tools pkg-config libssl-dev build-essential librocksdb-dev vim ca-certificates -y
RUN pip3 install toml-cli
RUN pip3 install toml
RUN pip3 install web3
RUN echo "export OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu" >>/etc/profile
RUN echo "export OPENSSL_INCLUDE_DIR=/usr/include/openssl" >>/etc/profile
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs >/root/rust_install.sh
RUN chmod +x /root/rust_install.sh
RUN /root/rust_install.sh --profile complete -y
RUN /bin/bash -c "source /root/.profile"
RUN /bin/bash -c "source /root/.bashrc"
RUN /bin/bash -c "source /root/.cargo/env"
RUN /bin/bash /root/.cargo/env
RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
RUN cargo install crm
RUN crm best
RUN rustup target add x86_64-unknown-linux-musl
RUN rustup install nightly
RUN rustup target add wasm32-unknown-unknown --toolchain nightly
RUN cargo install cargo-tarpaulin
RUN git clone https://github.com/FindoraNetwork/randao.git -b integration_test \
  && mv randao /root/ \
  && cd /root/randao  \
  && cargo build --release
COPY ./target/release/randao /bin/
COPY ./randao/config.json /root/

ENTRYPOINT ["/bin/randao", "--config", "/root/config.json"]