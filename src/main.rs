use git2::StatusOptions;

fn main() {
  use git2::Repository;

  let repo = match Repository::open("/Users/bas/github/api-cookbook") {
    Ok(repo) => repo,
    Err(e) => panic!("failed to open: {}", e),
  };
  repo.remotes().unwrap().iter().for_each(|remote| {
    let name = remote.unwrap();
    let remote = repo.find_remote(name).unwrap();
    println!("{} -> {:?}", name, remote.url());
  });
  repo.branches(None).unwrap().for_each(|branch| {
    let (branch, _) = branch.unwrap();
    let name = branch.name().unwrap().unwrap();
    if let Ok(upstream) = branch.upstream() {
      let upstream_name = upstream.name().unwrap().unwrap();
      println!("{} -> {}", name, upstream_name);
    } else {
      println!("{}", name);
    }
  });

  let mut binding = StatusOptions::new();
  let opts = binding.show(git2::StatusShow::IndexAndWorkdir).include_untracked(true);
  repo.statuses(Some(opts)).unwrap().iter().for_each(|entry| {
    let path = entry.path().unwrap();
    println!("{}", path);
  });
}
