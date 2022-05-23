#!/bin/bash -eu

# 1. Get current epoch
# 2. Get value of last epoch a bkcup has been done
# 3. Perform backup of all
# 4. Upload backup to s3
# 5. Save backup epoch done to file

source /env

if [[ "${NETWORK}" == testnet ]]; then
    NET_QUERY="--testnet-magic 1097911063"
else
    NET_QUERY="--mainnet"
fi
CURRENT_EPOCH=$(cardano-cli query tip ${NET_QUERY} | jq .epoch)

EPOCH_FILE=/epoch-file.txt
if [[ -f "${EPOCH_FILE}" ]]; then
    LAST_BACKUP_EPOCH=$(cat "${EPOCH_FILE}")
else
    LAST_BACKUP_EPOCH=0
fi

if [[ "${CURRENT_EPOCH}" -gt "${LAST_BACKUP_EPOCH}" ]]; then
    # Password is in env variable PGPASSFILE
    pg_dump \
        -h postgres \
        -U ${POSTGRES_USER:-oura} \
        -d ${POSTGRES_DB:-cardano} \
        > carp_db_${CARP_VERSION}_${CURRENT_EPOCH}.sql

    s3cmd put carp_db_${CARP_VERSION}_${CURRENT_EPOCH}.sql s3://${S3_BUCKET}/${S3_FOLDER}/${NETWORK}/
    rm -rf carp_db_${CARP_VERSION}_${CURRENT_EPOCH}.sql

    echo "${CURRENT_EPOCH}" > "${EPOCH_FILE}"
fi
