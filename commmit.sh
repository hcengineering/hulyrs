#!/bin/bash

clear

#git push origin --delete feature/insert_update
#git push origin --delete feature/insert_update_uuid

NAME="fix_rang"

git checkout -b feature/${NAME}
git add .
git commit -m "Fix rand 0.9.1 to 0.9 for compatibility"
git push origin feature/${NAME}
