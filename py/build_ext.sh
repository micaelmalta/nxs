#!/usr/bin/env bash
# Build the _nxs C extension without setuptools.
set -euo pipefail

cd "$(dirname "$0")"

PYINCLUDE=$(python3 -c "import sysconfig; print(sysconfig.get_path('include'))")
EXT_SUFFIX=$(python3 -c "import sysconfig; print(sysconfig.get_config_var('EXT_SUFFIX'))")

OUT="_nxs${EXT_SUFFIX}"

echo "Building $OUT"
echo "  Python include: $PYINCLUDE"

# -undefined dynamic_lookup lets the .so load without linking libpython on macOS
cc -O3 -Wall -Wextra -fPIC \
   -I"$PYINCLUDE" \
   -shared \
   -undefined dynamic_lookup \
   _nxs.c \
   -o "$OUT"

echo "Built $OUT"
