FROM fedora:latest

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

RUN chmod +x /usr/local/bin/hwatch-fedora-build-srpm

CMD ["/bin/bash"]
