#!/bin/bash
# Signs dist/RomP.app from inside the GUI session (gui/501), where the login
# keychain and Xcode's data-protection identities are actually accessible.
# Runs via com.rogenmagic.romp-sign LaunchAgent.
#
# The signature matters beyond Gatekeeper: RomP keeps the RomM sign-in in the
# keychain under service "Romp", and a keychain item is only readable without a
# password prompt by an app with the same code identity that created it. An
# ad-hoc rebuild changes that identity every time, so the fork is signed with the
# stable Apple Development certificate instead.
set -u

APP="/Users/admin/Development/RomP/dist/RomP.app"
LOG="/tmp/romp-sign.log"
STATUS="/tmp/romp-sign.status"

: > "$LOG"
echo "== whoami: $(whoami) ==" >> "$LOG"
echo "== identities ==" >> "$LOG"
security find-identity -v -p codesigning >> "$LOG" 2>&1

ID=$(security find-identity -v -p codesigning 2>/dev/null \
     | grep -E 'Apple Development|Developer ID Application|Mac Developer' \
     | head -1 | sed -E 's/^ *[0-9]+\) ([0-9A-Fa-f]{40}) .*/\1/')

if [ -z "$ID" ]; then
    echo "FAIL: no code-signing identity visible in the GUI session" >> "$LOG"
    echo "fail" > "$STATUS"
    exit 1
fi
echo "== signing with: $ID ==" >> "$LOG"

codesign --force --deep --sign "$ID" --timestamp=none "$APP" >> "$LOG" 2>&1
rc=$?
echo "== codesign rc=$rc ==" >> "$LOG"
codesign -dv "$APP" >> "$LOG" 2>&1
codesign --verify --deep --strict "$APP" >> "$LOG" 2>&1
echo "== verify rc=$? ==" >> "$LOG"

echo "$rc" > "$STATUS"
exit $rc
