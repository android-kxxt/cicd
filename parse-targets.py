#!/usr/bin/env python3
import sys
import json

targets = [target.strip() for target in sys.stdin.read().split(",")]
include = []

for target in targets:
    if target.endswith("+"):
        include.append(
            {
                "target": target[:-1],
                "sign": True,
                "unsigned": False,
            }
        )
    elif target.endswith("*"):
        include.append(
            {
                "target": target[:-1],
                "sign": True,
                "unsigned": True,
            }
        )
    else:
        include.append(
            {
                "target": target,
                "sign": False,
                "unsigned": True,
            }
        )

print(json.dumps({"include": include}))
