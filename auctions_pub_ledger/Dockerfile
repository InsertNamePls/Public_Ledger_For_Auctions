FROM debian:trixie
WORKDIR /home/

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
#RUN mkdir /home/ssh_keys && \
#    openssl genrsa -out /home/ssh_keys/private.pem 2048 && \
#    openssl rsa -in private.pem -outform PEM -pubout -out public.pem

#RUN curl https://sh.rustup.rs -sSf | bash -s -- -y
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | bash -s -- -y

COPY . /home/



