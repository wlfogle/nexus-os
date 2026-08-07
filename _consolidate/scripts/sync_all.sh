#!/bin/bash
# Path to your packages directory
BASE_DIR="/home/loufogle/nexus-os/packages"

for dir in "$BASE_DIR"/*/; do
    if [ -d "$dir/.git" ]; then
        echo "Syncing $dir..."
        cd "$dir" || continue

        # Add all changes
        git add .
        git commit -m "Automated sync: Overwriting remote with local state"

        # Force push the main branch to origin
        # Adjust 'main' if some repos use 'master'
        git push -f origin main || git push -f origin master

        echo "Finished syncing $dir"
    fi
done
