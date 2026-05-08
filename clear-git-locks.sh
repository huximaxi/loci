#!/usr/bin/env python3
"""
clear-git-locks.sh — remove stale git lock files in this repo.
Run if git complains: "Unable to create .git/index.lock: File exists"
Usage: python3 clear-git-locks.sh
"""
import os, sys

repo_root = os.path.dirname(os.path.abspath(__file__))
git_dir = os.path.join(repo_root, ".git")

locks = [
    os.path.join(git_dir, "index.lock"),
    os.path.join(git_dir, "HEAD.lock"),
    os.path.join(git_dir, "MERGE_HEAD.lock"),
    os.path.join(git_dir, "COMMIT_EDITMSG.lock"),
]

cleared = 0
for lock in locks:
    if os.path.exists(lock):
        stale = lock + ".stale"
        # increment suffix if stale already exists
        i = 1
        while os.path.exists(stale):
            stale = lock + f".stale{i}"
            i += 1
        try:
            os.rename(lock, stale)
            print(f"cleared: {os.path.basename(lock)}")
            cleared += 1
        except Exception as e:
            print(f"failed: {os.path.basename(lock)} -> {e}", file=sys.stderr)

if cleared == 0:
    print("no stale locks found")
else:
    print(f"done ({cleared} lock(s) cleared)")
