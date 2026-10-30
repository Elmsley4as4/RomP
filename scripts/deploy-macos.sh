#!/bin/bash
# Builds the app, signs it with the Apple Development certificate from inside the
# GUI session, and installs it on Vernon. One command, so a rebuild never lands
# ad-hoc signed (which would make RomP ask for the login keychain password).
set -euo pipefail

HOST="${ROMP_HOST:-richard@192.168.4.9}"
APP="$(cd "$(dirname "$0")/.." && pwd)/dist/RomP2.app"

# shellcheck disable=SC1091
source "$HOME/.cargo/env"
make -C "$(dirname "$0")/.." app

rm -f /tmp/romp-sign.status
launchctl kickstart -k gui/501/com.rogenmagic.romp-sign
for _ in $(seq 1 60); do
    [ -f /tmp/romp-sign.status ] && break
    sleep 1
done

status="$(cat /tmp/romp-sign.status 2>/dev/null || echo missing)"
if [ "$status" != "0" ]; then
    echo "signing failed (status: $status), see /tmp/romp-sign.log" >&2
    exit 1
fi
codesign --verify --deep --strict "$APP"
echo "== signature =="
codesign -dv "$APP" 2>&1 | grep -E "Identifier=|TeamIdentifier=" || true

rsync -a --delete "$APP/" "$HOST":/Applications/RomP2.app/
ssh "$HOST" 'pkill -x romp 2>/dev/null; sleep 1; open -a /Applications/RomP2.app'
echo "== installed on $HOST =="
