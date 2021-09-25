#!/usr/bin/env bash
cp $(dirname $0)/platuned.service /etc/systemd/system/platuned.service
systemctl daemon-reload
systemctl restart platuned