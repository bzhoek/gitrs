use anyhow::Result;
use clap::{Arg, Command};
use env_logger::Env;
use env_logger::Target::Stdout;
use git2::BranchType::{Local, Remote};
use git2::{Oid, StatusOptions};
use git2::{Repository, Status};
use log::{info, log, warn, Level};

fn main() -> Result<()> {
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
  if remotes.is_empty() {
    warn!("No remotes found");
  }
  remotes.iter().for_each(|remote| {
    let name = remote.unwrap();
    let remote = repo.find_remote(name).unwrap();
    let url = remote.url().unwrap();
    info!("  remote {} -> {}", name, url);
  });

  repo.branches(Some(Local))?.for_each(|branch| {
    let (branch, _) = branch.unwrap();
    let local_name = branch.name().unwrap().unwrap();
    let local_oid = branch.get().target().unwrap();
    if let Ok(upstream) = branch.upstream() {
      let upstream_name = upstream.name().unwrap().unwrap();
      let remote_oid = upstream.get().target().unwrap();
      let (local, remote) = repo.graph_ahead_behind(local_oid, remote_oid).unwrap();
      let level = get_log_level(local);
      log!(level, "upstream {} -> {}: {} > {}", local_name, upstream_name, local, remote);
    } else {
      warn!("  orphan {}", local_name);
      compare_orphan_to_remotes(&repo, local_oid).unwrap();
    }
  });

  let mut binding = StatusOptions::new();
  let opts = binding.show(git2::StatusShow::IndexAndWorkdir).include_untracked(true);
  let mut deleted = 0;
  let mut modified = 0;
  let mut untracked = 0;
  let mut unspecified = 0;
  repo.statuses(Some(opts))?.iter().for_each(|entry| {
    let path = entry.path().unwrap();
    match entry.status() {
      Status::WT_DELETED | Status::INDEX_DELETED => deleted += 1,
      Status::WT_MODIFIED | Status::INDEX_MODIFIED => modified += 1,
      Status::WT_NEW | Status::INDEX_NEW => untracked += 1,
      _ => unspecified += 1,
    }
    warn!("{:?}: {}", entry.status(), path);
  });
  let level = get_log_level(modified + deleted + untracked + unspecified);
  log!(level, "{} modified, {} deleted, {} untracked, {} unspecified\n", modified, deleted, untracked, unspecified);
  Ok(())
}

fn compare_orphan_to_remotes(repo: &Repository, local: Oid) -> Result<()> {
  repo.branches(Some(Remote))?
    .flatten()
    .flat_map(|(branch, _)| branch.get().resolve().ok().zip(Some(branch)))
    .for_each(|(reference, branch)| {
      let remote_name = branch.name().ok().flatten();
      let resolved_name = reference.name();
      let target = reference.target();
      remote_name.zip(resolved_name).zip(target)
        .map(|((remote_name, resolved_name), remote)| {
          let graph = repo.graph_ahead_behind(local, remote).ok();
          graph.inspect(|(local, remote)| {
            let level = get_log_level(*local);
            log!(level, "  remote {} -> {}: {} - {}", remote_name, resolved_name, local, remote)
          });
        });
    });
  Ok(())
}

fn get_log_level(local: usize) -> Level {
  if local > 0 {
    Level::Warn
  } else {
    Level::Info
  }
}
