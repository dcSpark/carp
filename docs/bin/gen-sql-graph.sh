#!/bin/bash

REQUIRED_PKG="graphviz"
PKG_OK=$(dpkg-query -W --showformat='${Status}\n' $REQUIRED_PKG|grep "install ok installed")
echo Checking for $REQUIRED_PKG: $PKG_OK
if [ "" = "$PKG_OK" ]; then
  echo "No $REQUIRED_PKG. Setting up $REQUIRED_PKG."
  sudo apt-get --yes install $REQUIRED_PKG
fi

pip install --user pipenv

# if this fails, try running `pipenv --rm && pipenv --clear` from the bin directory
pipenv install

pipenv shell "python ./sql_graphviz.py ./schema.sql | dot -Tsvg > ./sql-graph.svg && exit"