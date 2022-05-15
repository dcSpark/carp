FROM ubuntu:22.04

ENV CARDANO_NODE_URL=https://hydra.iohk.io/build/13065769/download/1/cardano-node-1.34.1-linux.tar.gz
ENV DEBIAN_FRONTEND=noninteractive
ENV TZ=UTC

RUN apt-get update && \
    apt-get -y install \
    postgresql-client \
    jq \
    wget \
    s3cmd \
    cron

RUN wget -O cardano-node.tar.gz \
    ${CARDANO_NODE_URL}
RUN tar -xvzf cardano-node.tar.gz ./cardano-cli && mv ./cardano-cli /bin/

COPY s3cfg /root/.s3cfg
COPY entrypoint.sh /
COPY backup.sh /

RUN crontab -l | { cat; echo "0 * * * * /backup.sh >> /var/log/cron.log 2>&1"; } | crontab -

ENTRYPOINT /entrypoint.sh
