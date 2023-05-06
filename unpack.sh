#!/bin/sh
T=`mktemp`
tail -c +104 "$0"|xz -dF raw --x86 --lzma2>$T
chmod +x $T
(sleep 3;rm $T)&exec $T
