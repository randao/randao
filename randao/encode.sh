#!/bin/bash

private_key=$1

if [ -z "$private_key" ]
then
  echo "Usage: $0 <private_key>"
  exit 1
fi

encrypted_key=$(echo -n "$private_key" | base64)

echo "$encrypted_key"
