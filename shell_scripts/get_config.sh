#!/bin/bash

# Extract service_port and grpc_ui_port from config.toml
SERVICE_PORT=$(grep "service_port" config.toml | awk -F'=' '{print $2}' | tr -d ' "')
GRPC_UI_PORT=$(grep "grpc_ui_port" config.toml | awk -F'=' '{print $2}' | tr -d ' "')

# Print both ports as a space-separated string
echo "$SERVICE_PORT $GRPC_UI_PORT"