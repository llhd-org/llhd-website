FROM ubuntu:18.04 as toolchain

ENV USER=root
ENV PATH=/root/.cargo/bin:$PATH

# Install dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
	curl

# Install Rust
RUN curl -sSf https://sh.rustup.rs | sh -s -- -y

# Install LLHD
RUN cargo install --git https://github.com/fabianschuiki/moore.git --tag v0.4.0 --force

# Configure working directory.
RUN mkdir -p /playground
WORKDIR /playground

# Configure the entry point which is run once the container is started.
ADD entrypoint.sh /root/
ENTRYPOINT ["/root/entrypoint.sh"]
