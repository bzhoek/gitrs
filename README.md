# Check Git change distribution 

```sh
cargo install --path .

# specific list
for repo in ~/.nix ~/.dotfiles ~/.brand; do gitrs $repo; done

export FROM_DIR=~/bzhoek
export FROM_DIR=~/github
export FROM_DIR=~/zilverline
for repo in $(find $FROM_DIR -maxdepth 1 -type d -exec realpath "{}" \; | sort -f); do \
  gitrs $repo; \
  done &> $(basename $FROM_DIR).log

# only git
for repo in $(find $FROM_DIR -name .git -type d -exec realpath "{}/.." \; | sort -f); do \
```