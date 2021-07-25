FROM fedora:34
RUN dnf update -y && dnf clean all -y
WORKDIR /usr/local/bin
COPY ./target/release/loyalty_microservice /usr/local/bin/loyalty_microservice
STOPSIGNAL SIGINT
ENTRYPOINT ["loyalty_microservice"]
