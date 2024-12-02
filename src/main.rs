use clap::{Arg, Command};
use git2::StatusOptions;
use git2::{Repository, Status};

fn main() {
  let args = Command::new("gitrs").arg(Arg::new("PATH").required(true)).get_matches();
  if !args.args_present() {
    eprintln!("{}", Command::new("my_app").render_help());
    std::process::exit(1);
  }

  let filepath = args.get_one::<String>("PATH").unwrap();
  println!("Opening repository at {}", filepath);

  let repo = match Repository::open(filepath) {
    Ok(repo) => repo,
    Err(e) => panic!("failed to open: {}", e),
  };

  let remotes = repo.remotes().unwrap();
  if remotes.len() == 0 {
    eprintln!("No remotes found");
  }
  remotes.iter().for_each(|remote| {
    let name = remote.unwrap();
    let remote = repo.find_remote(name).unwrap();
    println!("  remote {} -> {:?}", name, remote.url());
  });

  repo.branches(None).unwrap().for_each(|branch| {
    let (branch, _) = branch.unwrap();
    let name = branch.name().unwrap().unwrap();
    if let Ok(upstream) = branch.upstream() {
      let upstream_name = upstream.name().unwrap().unwrap();
      println!("upstream {} -> {}", name, upstream_name);
    } else {
      println!("  branch {}", name);
    }
  });

  let mut binding = StatusOptions::new();
  let opts = binding.show(git2::StatusShow::IndexAndWorkdir).include_untracked(true);
  let mut deleted = 0;
  let mut modified = 0;
  let mut untracked = 0;
  let mut unspecified = 0;
  repo.statuses(Some(opts)).unwrap().iter().for_each(|entry| {
    let path = entry.path().unwrap();
    match entry.status() {
      Status::WT_DELETED | Status::INDEX_DELETED => deleted += 1,
      Status::WT_MODIFIED | Status::INDEX_MODIFIED => modified += 1,
      Status::WT_NEW | Status::INDEX_NEW => untracked += 1,
      _ => unspecified += 1,
    }
    println!("{:?}: {}", entry.status(), path);
  });
  println!("{} modified, {} deleted, {} untracked, {} unspecified\n", modified, deleted, untracked, unspecified);
}
