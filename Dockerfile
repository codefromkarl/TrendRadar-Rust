FROM ubuntu:24.04

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && mkdir -p /app /config /data \
    && chown -R 1000:1000 /app /config /data

COPY target/release/trendradar /usr/local/bin/trendradar

USER 1000:1000
WORKDIR /app

ENTRYPOINT ["trendradar"]
