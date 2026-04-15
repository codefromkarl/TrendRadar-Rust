FROM ubuntu:24.04

RUN useradd --create-home --shell /usr/sbin/nologin trendradar \
    && mkdir -p /app /config /data \
    && chown -R trendradar:trendradar /app /config /data

COPY target/release/trendradar /usr/local/bin/trendradar

USER trendradar
WORKDIR /app

ENTRYPOINT ["trendradar"]
