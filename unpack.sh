#!/bin/sh
T=`mktemp`
tail -n+6 $0|xz -dcF raw --x86 --lzma2>$T
chmod +x $T
(sleep 3;rm $T)&exec $T
