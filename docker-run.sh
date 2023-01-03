#!/bin/bash
nohup /bin/randao --config /root/.randao/config/config.json > /root/findora-randao.log 2>&1 &
/bin/bash -c "while true;do echo hello;sleep 50000;done"