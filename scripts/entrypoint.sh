#!/bin/bash
# Tools Service Entrypoint
# Starts: Next.js frontend (port 3000), Rust gateway (port 3001), Workers (NATS consumers)

set -e

# ── 1. Find binaries ──

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

# ── 2. Start Rust Gateway (port 3001) ──

echo "Starting tools-gateway ($GATEWAY_BIN) on port 3001..."
"$GATEWAY_BIN" &
GATEWAY_PID=$!

sleep 1

# ── 3. Start Rust Workers ──

echo "Starting tools-workers ($WORKER_BIN)..."
"$WORKER_BIN" &
WORKER_PID=$!

# ── 4. Start Next.js Frontend (port 3000) ──

if command -v bun &>/dev/null && [ -f /app/node_modules/.bin/next ]; then
  echo "Starting Next.js frontend on port 3000..."
  cd /app
  NODE_ENV=production RUST_GATEWAY_URL=http://localhost:3001 \
    bun run next start --port 3000 &
  NEXT_PID=$!
  echo "Next.js frontend started (PID: $NEXT_PID)"
elif command -v node &>/dev/null && [ -f /app/node_modules/.bin/next ]; then
  echo "Starting Next.js frontend on port 3000..."
  cd /app
  NODE_ENV=production RUST_GATEWAY_URL=http://localhost:3001 \
    node /app/node_modules/.bin/next start --port 3000 &
  NEXT_PID=$!
  echo "Next.js frontend started (PID: $NEXT_PID)"
else
  echo "WARNING: Node.js/bun not found, frontend will not be served"
  NEXT_PID=""
fi

# ── 5. Graceful shutdown ──

trap "echo 'Shutting down...'; kill $GATEWAY_PID $WORKER_PID $NEXT_PID 2>/dev/null; wait; exit 0" SIGINT SIGTERM

# Wait for any process to exit
if [ -n "$NEXT_PID" ]; then
  wait -n $GATEWAY_PID $WORKER_PID $NEXT_PID
else
  wait -n $GATEWAY_PID $WORKER_PID
fi

# If one exits, kill the others
kill $GATEWAY_PID $WORKER_PID $NEXT_PID 2>/dev/null
exit 1