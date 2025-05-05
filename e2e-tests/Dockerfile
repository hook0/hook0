FROM grafana/k6:0.59.0

USER root
RUN apk add --no-cache bash

USER k6

ENV ENABLE_SETUP=false \
    VUS=1 \
    ITERATIONS=1 \
    DURATION=1m

WORKDIR /app
COPY src/ ./src
COPY run-main-tests.sh ./

ENTRYPOINT ["/app/run-main-tests.sh"]
