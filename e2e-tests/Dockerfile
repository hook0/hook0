FROM grafana/k6:0.56.0

USER root

RUN apk add --no-cache bash

USER k6

ENV ENABLE_SETUP=false \
    VUS=10 \
    ITERATIONS=100 \
    DURATION=1m

COPY src/ ./src
COPY run-main-tests.sh ./

ENTRYPOINT ["/bin/sh", "-c"]
CMD ["bash run-main-tests.sh"]
