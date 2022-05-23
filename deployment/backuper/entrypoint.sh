#!/bin/bash

# ensure there are no duplicate entries
sed -i '/access_key =/d' /root/.s3cfg
sed -i '/secret_key =/d' /root/.s3cfg

echo "access_key = ${AWS_ACCESS_KEY_ID:-empty}" >> /root/.s3cfg
echo "secret_key = ${AWS_SECRET_ACCESS_KEY:-empty}" >> /root/.s3cfg

echo "NETWORK=${NETWORK}" > /env

echo "POSTGRES_USER=${POSTGRES_USER}" >> /env
echo "POSTGRES_PASSWORD=${POSTGRES_PASSWORD}" >> /env
echo "POSTGRES_HOST=${POSTGRES_HOST}" >> /env
echo "POSTGRES_PORT=${POSTGRES_PORT}" >> /env
echo "POSTGRES_DB=${POSTGRES_DB}" >> /env

echo "DATABASE_URL=postgresql://${POSTGRES_USER}:${POSTGRES_PASSWORD}@${POSTGRES_HOST}:${POSTGRES_PORT}/${POSTGRES_DB}" >> /env

echo "S3_BUCKET=${S3_BUCKET}" >> /env
echo "S3_FOLDER=${S3_FOLDER}" >> /env
echo "CARP_VERSION=${CARP_VERSION}" >> /env

touch /var/log/cron.log
cron && tail -f /var/log/cron.log
