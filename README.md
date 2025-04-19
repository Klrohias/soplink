# soplink
![soplink](https://socialify.git.ci/Klrohias/soplink/image?language=1&name=1&owner=1&theme=Auto)
A script that simplifies SOP(Single-Object-Prelink) linking

# Why?
When creating static libs for iOS or other platforms that can only use static linking, symbol conflicts often occur, so I wrote this tool to create SOP(Single-Object-Prelink) static libs, which hide useless symbols and avoid symbol conflicts.
﻿
Also, static libs are usually large, and making them SOP static libs can reduce them to a comfortable size.

# How to use?
```shell
# Use -s to specify which symbols to preserve, wildcards are supported
./soplink /my_workspace/libuniasset.a -s "Uniasset_*" -s "_Uniasset_*" -F -o libuniasset_trimed.a
```

```shell
# Use -l to specify a file to tell soplink which symbols to preserve
cat symbols.txt
# 
# # Comments are allowed with `#` starting lines
# _Uniasset_*
# Uniasset_*
#
./soplink /my_workspace/libuniasset.a -l symbols.txt -F -o libuniasset_trimed.a
```

# How much can the size be reduced to?
In my case, on macOS, a debug-build library with a size 115M can be reduce to 38M, and a release-build library reduce from 29M to 14M.

