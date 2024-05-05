FROM debian:trixie
WORKDIR /home/

# Install necessary packages
RUN apt-get update -y && apt-get install -y \
  build-essential \
  curl \
  cargo \
  net-tools \
  pkg-config \
  libssl-dev \
  vim \
  iputils-ping \
  ssh \
  protobuf-compiler \
  python3 \
  python3-pip

# Install Rust
RUN curl https://sh.rustup.rs -sSf | bash -s -- -y

# Copy your Flask application
COPY . /home/

# Install Python dependencies
RUN pip3 install -r requirements.txt

# Expose the port Flask is running on
EXPOSE 5000

# Command to run the Flask application
CMD ["python3", "app.py"]
