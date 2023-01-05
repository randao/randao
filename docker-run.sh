#!/bin/bash
nohup /bin/randao --config /tmp/.randao/config/config.json > /tmp/findora-randao.log 2>&1 &
/bin/bash -c "while true;do echo hello;sleep 50000;done"