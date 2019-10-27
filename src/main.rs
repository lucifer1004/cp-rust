use std::fs::{copy as fcopy, File};
use std::io::{copy, Error};
use std::process::{Command, Stdio};

use clap::Clap;
use dotenv::dotenv;

mod codeforces;
mod webdriver;

/// Handy commands for competitive programming in rust.
#[derive(Clap)]
#[clap()]
struct Opts {
  #[clap(subcommand)]
  subcmd: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
  /// Create a new source file from a template file.
  #[clap(name = "new")]
  New {
    /// Names of the files to be created.
    file_names: Vec<String>,
    /// Select template to use.
    #[clap(short = "t", default_value = "default")]
    template: String,
  },

  /// Execute a program with an input file.
  #[clap(name = "exec")]
  Exec {
    /// Name of the file to be executed.
    file_name: String,
    /// Select test input.
    #[clap(short = "t", default_value = "test.in")]
    test_file: String,
  },

  /// Add and commit a source file.
  #[clap(name = "commit")]
  Commit {
    /// Name of the file to be added and created.
    file_name: String,
  },

  /// Submit to Codeforces
  #[clap(name = "submit")]
  Submit {
    /// Name of the file to be submitted.
    file_name: String,
  },

  /// Query Codeforces problems.
  #[clap(name = "problem")]
  Problem {
    /// Names of the tags to be queried.
    #[clap(short = "t")]
    tag: Vec<String>,
    /// Names of the tags to be queried.
    #[clap(short = "p")]
    problemset_name: Option<String>,
  },

  /// Query Codeforces users.
  #[clap(name = "user")]
  User {
    /// Names of the users to be queried.
    #[clap(short = "n")]
    username: Vec<String>,
  },

  /// Query Codeforces blogs.
  #[clap(name = "blog")]
  Blog {
    /// Number of the blog.
    #[clap(short = "n")]
    number: u32,
  },
}

#[tokio::main]
async fn main() -> Result<(), Error> {
  dotenv().ok();
  let opts: Opts = Opts::parse();

  match opts.subcmd {
    SubCommand::New {
      file_names,
      template,
    } => {
      for file_name in file_names {
        fcopy(
          format!("src/templates/{}.rs", template),
          format!("src/bin/{}.rs", file_name),
        )
        .expect("failed to create file");
      }
    }

    SubCommand::Exec {
      file_name,
      test_file,
    } => {
      let mut child = Command::new("sh")
        .arg("-c")
        .arg(format!("cargo run --bin {}", file_name))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to execute file");

      let mut file = File::open(test_file)
        .ok()
        .expect("failed to open input file");
      copy(&mut file, child.stdin.as_mut().unwrap()).expect("failed to copy input");
      let output = child.wait_with_output().unwrap();
      let output = String::from_utf8(output.stdout).unwrap();
      println!("{}", output);
    }

    SubCommand::Commit { file_name } => {
      Command::new("sh").arg("-c").arg(format!(
        "git add ./src/bin/{}.rs && git commit -m \"{}\"",
        file_name, file_name
      ));
    }

    SubCommand::Submit { file_name } => {
      let mut cf = webdriver::init().await;
      cf.login().await.expect("cannot login");
      cf.submit(file_name).await.expect("submit error");
      cf.exit().await.expect("exit with error");
    }

    SubCommand::Problem {
      tag,
      problemset_name,
    } => {
      let tags = match tag.len() {
        0 => None,
        _ => Some(tag.join(";").to_string()),
      };
      codeforces::get_problemset_problems(tags, problemset_name)
        .expect("failed to query problemset.problems");
    }

    SubCommand::User { username } => {
      let handles = username.join(";");
      codeforces::get_user_info(&handles).expect("failed to query user.info");
    }

    SubCommand::Blog { number } => {
      codeforces::get_blog_entry(number).expect("failed to query blogEntry.view");
    }
  }

  Ok(())
}
