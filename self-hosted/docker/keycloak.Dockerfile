ARG KEYCLOAK_VERSION=22.0

FROM redhat/ubi9 as ubi-micro-build
RUN mkdir -p /mnt/rootfs
RUN dnf install --installroot /mnt/rootfs curl jq --releasever 9 --setopt install_weak_deps=false --nodocs -y; dnf --installroot /mnt/rootfs clean all

FROM keycloak/keycloak:${KEYCLOAK_VERSION}
COPY --from=ubi-micro-build /mnt/rootfs /
COPY self-hosted/configure-keycloak.sh /opt/
COPY self-hosted/docker/keycloak/run.sh /opt/

ENV KC_DB=postgres
ENV KC_HTTP_ENABLED=true
ENV KC_HEALTH_ENABLED=true
ENV KC_HOSTNAME_STRICT=false

RUN /opt/keycloak/bin/kc.sh build
ENTRYPOINT ["/opt/run.sh"]
