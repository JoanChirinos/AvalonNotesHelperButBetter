#!/usr/bin/env bash
set -euo pipefail

# Back up the anh3 SQLite DB, WAL-safe, with integrity check + retention.
# Usage: backup-db.sh [label]   (label defaults to "daily"; deploy.sh passes "predeploy")

DB="$HOME/AvalonNotesHelperButBetter/backend/avalon.db"
BACKUP_DIR="$HOME/AvalonNotesHelperButBetter-backups"
RETENTION=30
LABEL="${1:-daily}"
STAMP="$(date +%Y-%m-%d_%H%M%S)"
DEST="$BACKUP_DIR/avalon_${STAMP}_${LABEL}.db"

mkdir -p "$BACKUP_DIR"

if [ ! -f "$DB" ]; then
  echo "$(date): ERROR: DB not found at $DB" >&2
  exit 1
fi

# Online backup: VACUUM INTO is WAL-safe, atomic, and compacts the copy.
/usr/bin/sqlite3 "$DB" "VACUUM INTO '$DEST';"

# Verify the copy before we trust it
if ! /usr/bin/sqlite3 "$DEST" "PRAGMA integrity_check;" | grep -q "^ok$"; then
  echo "$(date): ERROR: integrity check FAILED for $DEST" >&2
  rm -f "$DEST"
  exit 1
fi

gzip -f "$DEST"
echo "$(date): backup OK -> ${DEST}.gz"

# Prune: keep newest $RETENTION, delete the rest
ls -1t "$BACKUP_DIR"/avalon_*.db.gz 2>/dev/null | tail -n +$((RETENTION + 1)) | while read -r old; do
  rm -f "$old"
  echo "$(date): pruned $old"
done
