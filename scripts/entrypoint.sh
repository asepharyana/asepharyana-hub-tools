#!/bin/bash
# Tools Service Entrypoint
# Starts both the Gateway (Axum HTTP server) and Workers (NATS consumers)

set -e

# Find gateway binary (named tools-gateway or in gateway/ subdir)
GATEWAY_BIN=""
for candidate in /app/gateway/tools-gateway /app/gateway /app/tools-gateway /app/target/release/tools-gateway; do
  if [ -f "$candidate" ] && [ -x "$candidate" ]; then
    GATEWAY_BIN="$candidate"
    break
  fi
done

if [ -z "$GATEWAY_BIN" ]; then
  echo "ERROR: tools-gateway binary not found"
  ls -la /app/ 2>/dev/null | head -5
  exit 1
fi

# Find workers binary
WORKER_BIN=""
for candidate in /app/workers/tools-workers /app/workers /app/tools-workers /app/target/release/tools-workers; do
  if [ -f "$candidate" ] && [ -x "$candidate" ]; then
    WORKER_BIN="$candidate"
    break
  fi
done

if [ -z "$WORKER_BIN" ]; then
  echo "ERROR: tools-workers binary not found"
  ls -la /app/ 2>/dev/null | head -5
  exit 1
fi

echo "Starting tools-gateway ($GATEWAY_BIN)..."
"$GATEWAY_BIN" &
GATEWAY_PID=$!

sleep 1

echo "Starting tools-workers ($WORKER_BIN)..."
"$WORKER_BIN" &
WORKER_PID=$!

# Handle graceful shutdown
trap "echo 'Shutting down...'; kill $GATEWAY_PID $WORKER_PID 2>/dev/null; wait; exit 0" SIGINT SIGTERM

# Wait for either process to exit
wait -n $GATEWAY_PID $WORKER_PID

# If one exits, kill the other
kill $GATEWAY_PID $WORKER_PID 2>/dev/null
exit 1