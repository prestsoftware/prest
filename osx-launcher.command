#!/usr/bin/env bash
cat "$0" | sed '1,6d' > /tmp/prest
chmod +x /tmp/prest
/tmp/prest
rm /tmp/prest
exit 0
