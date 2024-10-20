#!/bin/sh

api=https://api.github.com/repos/ivy-rew/eink-vnc/actions/artifacts

artifacts=$(curl -L \
  -H "Accept: application/vnd.github+json" \
  -H "Authorization: Bearer ${gh_token}" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  $api?per_page=1)

latestId=$(echo $artifacts | grep -m 1 -o -E '\"id\": [0-9]+' | head -1 | grep -o -E "[0-9]+")

echo ${latestId}

rm *.zip
curl -o latest.zip -L \
  -H "Accept: application/vnd.github+json" \
  -H "Authorization: Bearer ${gh_token}" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  $api/${latestId}/zip

rm einkvnc
unzip latest.zip