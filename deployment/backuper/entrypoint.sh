#!/bin/bash

# ensure there are no duplicate entries
sed -i '/access_key =/d' /root/.s3cfg
sed -i '/secret_key =/d' /root/.s3cfg

echo "access_key = ${AWS_ACCESS_KEY_ID:-empty}" >> /root/.s3cfg
echo "secret_key = ${AWS_SECRET_ACCESS_KEY:-empty}" >> /root/.s3cfg

touch /var/log/cron.log
cron && tail -f /var/log/cron.log
