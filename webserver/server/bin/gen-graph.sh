# sudo apt install graphviz
# pip install --user pipenv
# (in this folder) pipenv install
pipenv shell "python ./sql_graphviz.py ./schema.sql | dot -Tpng > ./graph.png && exit"