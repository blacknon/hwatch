ARG FEDORA_VERSION=42
FROM fedora:${FEDORA_VERSION}

RUN dnf install -y \
    bash \
    bash-completion \
    cargo-rpm-macros \
    fedpkg \
    git \
    rpm-build \
    tar \
    which \
 && dnf clean all

WORKDIR /work

COPY package/fedora/build-srpm.sh /usr/local/bin/hwatch-fedora-build-srpm
COPY package/fedora/build-with-mock.sh /usr/local/bin/hwatch-fedora-build-with-mock

RUN chmod +x /usr/local/bin/hwatch-fedora-build-srpm /usr/local/bin/hwatch-fedora-build-with-mock

CMD ["/bin/bash"]
