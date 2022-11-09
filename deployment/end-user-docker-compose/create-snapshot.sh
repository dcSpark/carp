#!/bin/bash

export PWD=$(pwd)

echo "Checking if rsnapshot is installed"

which rsnapshot

if [ "$?" -ne "0" ]
then
echo "rsnapshot not found. exiting"
exit 1
fi


ENV=${1:-.env}

if [ ! -f $ENV ]
then 
echo "Environment file not found"
exit 1
fi

echo "Generating rsnapshot.conf from .env file"
# Reading env file and export values for envsubst
export $(cat $ENV | grep -v ^\# | xargs)

if [ "$SNAPSHOTS_TOP_DIR" == "$TOP_DIR" ] || [ -z ${SNAPSHOTS_TOP_DIR+x} ]
then
  SNAPSHOT_TOP_DIR=$TOP_DIR/snapshots
fi


TOP_DIR=$(realpath $TOP_DIR)
SNAPSHOTS_TOP_DIR=$(realpath $SNAPSHOTS_TOP_DIR)

mkdir -p $SNAPSHOTS_TOP_DIR


# Replace Variables
temp_rsnapshot_config=$(mktemp)
envsubst < rsnapshot.conf.template > ${temp_rsnapshot_config}

echo "Shutting down environment"
docker-compose down 

echo "Creating a snapshot"
rsnapshot -c ${temp_rsnapshot_config} ${INSTANCE}

echo "Bringing up environment"
docker-compose up -d

echo "Generating tar file"

TAR_DATE=`date +%Y%m%d`
TAR_TIME=`date +%H%M`


tar -czf carp-${INSTANCE}-${TAR_DATE}-${TAR_TIME}.tar.gz -C ${SNAPSHOTS_TOP_DIR}/data/${INSTANCE}.0/localhost${TOP_DIR}/ .

echo "Sending file to AWS"

