#!/usr/bin/env bash
# ==============================================================================
# SENTRY-EDGE // Daemon Management Script
# Autonomous 2-3 Day Real-World Endurance Soak Supervisor
# ==============================================================================

SENTRY_BIN="$HOME/.local/bin/sentry-edge"
LOG_DIR="$HOME/.config/sentry/logs"
PID_FILE="$HOME/.config/sentry/sentry.pid"
STDOUT_LOG="$LOG_DIR/sentry_daemon.stdout"

mkdir -p "$LOG_DIR"

case "$1" in
  start)
    if [ -f "$PID_FILE" ] && kill -0 $(cat "$PID_FILE") 2>/dev/null; then
      echo "⚠️ Sentry daemon is already running (PID: $(cat "$PID_FILE"))"
      exit 0
    fi
    echo "🚀 Launching Sentry-Edge Hardware Sentinel Daemon in background..."
    nohup "$SENTRY_BIN" run > "$STDOUT_LOG" 2>&1 &
    echo $! > "$PID_FILE"
    echo "✔ Sentry daemon started (PID: $!)"
    echo "📄 Stdout Log: $STDOUT_LOG"
    ;;

  stop)
    if [ -f "$PID_FILE" ]; then
      PID=$(cat "$PID_FILE")
      echo "🛑 Stopping Sentry daemon (PID: $PID)..."
      kill "$PID" 2>/dev/null || true
      rm -f "$PID_FILE"
      echo "✔ Sentry daemon stopped."
    else
      echo "ℹ️ No active Sentry PID file found."
    fi
    ;;

  status)
    if [ -f "$PID_FILE" ] && kill -0 $(cat "$PID_FILE") 2>/dev/null; then
      PID=$(cat "$PID_FILE")
      echo "🟢 Sentry Daemon is RUNNING (PID: $PID)"
      ps -p "$PID" -o pid,user,%cpu,%mem,rss,etime,command
    else
      echo "🔴 Sentry Daemon is NOT running."
    fi
    ;;

  logs)
    shift
    FOLLOW=""
    TAIL="20"
    while [[ $# -gt 0 ]]; do
      case "$1" in
        -f|--follow)
          FOLLOW="-f"
          shift
          ;;
        *)
          if [[ "$1" =~ ^[0-9]+$ ]]; then
            TAIL="$1"
          fi
          shift
          ;;
      esac
    done
    "$SENTRY_BIN" logs $FOLLOW --tail "$TAIL"
    ;;

  stats)
    "$SENTRY_BIN" logs --stats
    ;;

  verify)
    "$SENTRY_BIN" logs --verify-chain
    ;;

  report)
    FORMAT="${2:-html}"
    if [ -n "$3" ]; then
      REPORT_OUT="$3"
      "$SENTRY_BIN" report --format "$FORMAT" --output "$REPORT_OUT"
    elif [[ "$2" == *"."* ]] || [[ "$2" == *"/"* ]]; then
      REPORT_OUT="$2"
      "$SENTRY_BIN" report --output "$REPORT_OUT"
    else
      REPORT_OUT="$HOME/sentry_endurance_report.$FORMAT"
      "$SENTRY_BIN" report --format "$FORMAT" --output "$REPORT_OUT"
    fi
    ;;

  *)
    echo "Usage: sentry-daemon.sh {start|stop|status|logs [-f] [N]|stats|verify|report [FORMAT] [OUT_PATH]}"
    exit 1
    ;;
esac
