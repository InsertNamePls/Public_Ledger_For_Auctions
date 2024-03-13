FROM debian:bookworm-slim

RUN apt-get update -y
RUN apt-get install -y \
    build-essential \
    curl \
    cargo \
    net-tools \
    pkg-config \
    libssl-dev \
    vim \
    iputils-ping \
    ssh
RUN mkdir /home/ssh_keys && \ 
    ssh-keygen -t rsa -b 4096 -f /home/ssh_keys/idrsa -C "test" -q -N ""
RUN curl https://sh.rustup.rs -sSf | bash -s -- -y
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | bash -s -- -y

#CMD ["mkdir /home/public_ledger_for_auctions"]

ADD . /home/
