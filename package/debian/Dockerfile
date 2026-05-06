FROM debian:sid

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y --no-install-recommends \
    bash \
    bash-completion \
    build-essential \
    ca-certificates \
    cargo \
    debhelper \
    devscripts \
    dh-cargo \
    dh-sequence-cargo \
    dpkg-dev \
    dput-ng \
    equivs \
    gcc \
    git \
    gnupg2 \
    libstd-rust-dev \
    lintian \
    python3 \
    rsync \
    rustc \
 && rm -rf /var/lib/apt/lists/*

WORKDIR /work

COPY package/debian/build-and-upload.sh /usr/local/bin/hwatch-debian-build

RUN chmod +x /usr/local/bin/hwatch-debian-build

CMD ["/bin/bash"]
