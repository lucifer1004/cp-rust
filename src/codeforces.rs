use std::{collections::HashMap, env, error::Error, fs, thread, time};

use data_encoding::HEXUPPER;
use fantoccini::Locator;
use rand::distributions::Alphanumeric;
use rand::prelude::*;
use reqwest;
use ring::digest::{Context, SHA512};
use serde::{Deserialize, Serialize};

use crate::webdriver::Session;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct User {
  handle: String,
  email: Option<String>,
  vk_id: Option<String>,
  open_id: Option<String>,
  first_name: Option<String>,
  last_name: Option<String>,
  country: Option<String>,
  city: Option<String>,
  organization: Option<String>,
  contribution: i32,
  rank: String,
  rating: i32,
  max_rank: String,
  max_rating: i32,
  last_online_time_seconds: u32,
  registration_time_seconds: u32,
  friend_of_count: u32,
  avatar: String,
  title_photo: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Problem {
  contest_id: Option<u32>,
  problemset_name: Option<String>,
  index: String,
  name: String,
  r#type: String,
  points: Option<f64>,
  rating: Option<u32>,
  phones: Option<Vec<String>>,
  tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProblemStatistics {
  contest_id: Option<u32>,
  index: String,
  solved_count: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProblemQueryResult {
  problems: Vec<Problem>,
  problem_statistics: Vec<ProblemStatistics>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BlogEntry {
  id: u32,
  original_locale: String,
  creation_time_seconds: u32,
  author_handle: String,
  title: String,
  content: String,
  locale: String,
  modification_time_seconds: u32,
  allow_view_history: bool,
  tags: Vec<String>,
  rating: i32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CodeforcesResponse<T> {
  status: String,
  result: T,
}

impl Session {
  pub async fn login(&mut self) -> Result<(), fantoccini::error::CmdError> {
    if let Some(ref mut c) = self.client {
      c.goto("https://codeforces.com/enter").await?;

      let mut f = c.form(Locator::Css("#enterForm")).await?;
      f.set_by_name("handleOrEmail", &env::var("CODEFORCES_EMAIL").unwrap())
        .await?;
      f.set_by_name("password", &env::var("CODEFORCES_PASSWORD").unwrap())
        .await?;
      f.submit().await?;
    }

    Ok(())
  }

  pub async fn submit(&mut self, problem: String) -> Result<(), fantoccini::error::CmdError> {
    if let Some(ref mut c) = self.client {
      c.goto("https://codeforces.com/problemset/submit").await?;

      let mut f = c.form(Locator::Css(".submit-form")).await?;
      f.set_by_name("submittedProblemCode", &problem.to_uppercase())
        .await?;

      let lang = c.find(Locator::Css("select[name='programTypeId']")).await?;
      lang.select_by_value("49").await?;

      let filename = format!("src/bin/{}.rs", problem);
      let code = fs::read_to_string(filename).expect("failed to read from file");
      let mut code_area = c.find(Locator::Css("#sourceCodeTextarea")).await?;
      code_area.send_keys(&code).await?;

      f.submit().await?;
      thread::sleep(time::Duration::from_millis(10000));
    }
    Ok(())
  }

  pub async fn exit(&mut self) -> Result<(), fantoccini::error::CmdError> {
    if let Some(ref mut c) = self.client {
      c.close().await?
    }
    Ok(())
  }
}

pub fn get_problemset_problems(
  tags: Option<String>,
  problemset_name: Option<String>,
) -> Result<(), Box<dyn Error>> {
  let query = "https://codeforces.com/api/problemset.problems";
  let mut params = HashMap::new();
  if let Some(tags) = tags {
    params.insert("tags", tags);
  }
  if let Some(problemset_name) = problemset_name {
    params.insert("problemsetName", problemset_name);
  }

  let client = reqwest::Client::new();

  let resp: CodeforcesResponse<ProblemQueryResult> =
    client.get(query).form(&params).send()?.json()?;

  println!("{:?}", resp);
  Ok(())
}

pub fn get_user_info(handles: &str) -> Result<(), Box<dyn Error>> {
  let resp: CodeforcesResponse<Vec<User>> = reqwest::get(&format!(
    "https://codeforces.com/api/user.info?handles={}",
    handles
  ))?
  .json()?;
  println!("{:?}", resp);
  Ok(())
}

pub fn get_blog_entry(number: u32) -> Result<(), Box<dyn Error>> {
  let resp: CodeforcesResponse<BlogEntry> = reqwest::get(&format!(
    "https://codeforces.com/api/blogEntry.view?blogEntryId={}",
    number
  ))?
  .json()?;
  println!("{:?}", resp);
  Ok(())
}

pub fn sign(method: String) -> String {
  let cf_key = env::var("CODEFORCES_API_KEY").unwrap();
  let cf_secret = env::var("CODEFORCES_API_SECRET").unwrap();
  let rng = rand::thread_rng();
  let rand_string: String = rng.sample_iter(Alphanumeric).take(6).collect();
  let current_time = time::SystemTime::now()
    .duration_since(time::UNIX_EPOCH)
    .expect("failed to get system time");
  let raw = format!(
    "{}/{}?apiKey={}&time={:?}#{}",
    &rand_string,
    &method,
    &cf_key,
    current_time.as_secs(),
    &cf_secret
  );

  let mut context = Context::new(&SHA512);
  context.update(raw.as_bytes());
  let hash = HEXUPPER.encode(context.finish().as_ref());
  format!(
    "https://codeforces.com/api/{}?apiKey={}&time={:?}&apiSig={}{}",
    &method,
    &cf_key,
    current_time.as_secs(),
    &rand_string,
    &hash
  )
}

#[cfg(test)]
mod tests {
  use super::*;
  use dotenv::dotenv;

  #[test]
  fn test_sign() {
    dotenv().ok();
    let digest = sign("problemset.problems".to_string());
    println!("{:?}", digest);
  }
}
