# Check Git change distribution 

```sh
cargo install --path .

for repo in ~/.nix ~/.dotfiles ~/.brand; do target/debug/gitrs $repo; done

for repo in $(find ~/bzhoek -name .git -type d -exec realpath "{}/.." \; | sort -f); do target/debug/gitrs $repo; done
```