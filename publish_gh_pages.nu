#!/usr/bin/env nu

# TODO: add a better check, to check that js and wasm exist
# Check if the directory exists
let wasm_path_type = ('wasm' | path type -c [ name ])
if $wasm_path_type == "" {
  echo "The directory 'wasm' does not exist."
  exit 1
}

# (re) Create a gh-pages branch
try {
  git branch -D gh-pages
}

git checkout --orphan gh-pages

# Add all files to the index
git reset
git add -f ./wasm/*

# Commit the changes
git commit -m "Deploying to GitHub Pages"

# Push the changes to the gh-pages branch
git push -f origin gh-pages


git clean -f -d
git checkout main
git checkout gh-pages -- ./wasm
git reset
git branch -D gh-pages

# Print a success message
echo "Successfully deployed to GitHub Pages!"