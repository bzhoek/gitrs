use clap::{Arg, Command};
use env_logger::Env;
use env_logger::Target::Stdout;
use git2::BranchType::{Local, Remote};
use git2::StatusOptions;
use git2::{Repository, Status};
use log::{info, warn};

fn main() {
  let args = Command::new("gitrs").arg(Arg::new("PATH").required(true)).get_matches();
  env_logger::Builder::from_env(Env::default().default_filter_or("info")).target(Stdout).init();
  if !args.args_present() {
    eprintln!("{}", Command::new("my_app").render_help());
    std::process::exit(1);
  }

  let filepath = args.get_one::<String>("PATH").unwrap();
  info!("Opening repository at {}", filepath);

  let repo = match Repository::open(filepath) {
    Ok(repo) => repo,
    Err(e) => panic!("failed to open: {}", e),
  };

  let remotes = repo.remotes().unwrap();
  if remotes.len() == 0 {
    warn!("No remotes found");
  }
  remotes.iter().for_each(|remote| {
    let name = remote.unwrap();
    let remote = repo.find_remote(name).unwrap();
    let url = remote.url().unwrap();
    info!("  remote {} -> {}", name, url);
  });

  repo.branches(Some(Local)).unwrap().for_each(|branch| {
    let (branch, _) = branch.unwrap();
    let name = branch.name().unwrap().unwrap();
    let local = branch.get().target().unwrap();
    if let Ok(upstream) = branch.upstream() {
      let upstream_name = upstream.name().unwrap().unwrap();
      info!("upstream {} -> {}", name, upstream_name);
      let remote = upstream.get().target().unwrap();
      let (local, remote) = repo.graph_ahead_behind(local, remote).unwrap();
      info!("upstream {} -> {}", local, remote);
    } else {
      warn!("  branch {}", name);
      repo.branches(Some(Remote)).unwrap().for_each(|remote| {
        let (remote, _type) = remote.unwrap();
        warn!("  remote {:?}, {:?}", remote.name(), remote.is_head());
        if let Ok(resolved) = remote.get().resolve() {
          let remote = resolved.target();
          let graph = remote.map(|remote| repo.graph_ahead_behind(local, remote).ok()).flatten();
          graph.inspect(|(local, remote)| info!("upstream {} -> {}", local, remote));
        }
        if let Some(remote) = remote.get().target() {
          let (local, remote) = repo.graph_ahead_behind(local, remote).unwrap();
          info!("upstream {} -> {}", local, remote);
        }
      });
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
    warn!("{:?}: {}", entry.status(), path);
  });
  info!("{} modified, {} deleted, {} untracked, {} unspecified\n", modified, deleted, untracked, unspecified);
}
