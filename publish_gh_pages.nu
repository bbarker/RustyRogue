#!/usr/bin/env nu

# Check if the directory exists
let wasm_path_type = ('wasm' | path type -c [ name ])
if $wasm_path_type == "" {
  echo "The directory 'wasm' does not exist."
  exit 1
}

# Change directory to the directory
cd "wasm"

# (re) Create a gh-pages branch
git branch -D gh-pages
git checkout -b gh-pages

# Add all files to the index
git add .

# Commit the changes
git commit -m "Deploying to GitHub Pages"

# Push the changes to the gh-pages branch
git push -f origin gh-pages

# Print a success message
echo "Successfully deployed to GitHub Pages!"