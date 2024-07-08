use std::{
    collections::{BTreeMap, HashMap},
    env,
    error::Error,
    fs, thread, time,
};

use data_encoding::HEXLOWER;
use fantoccini::Locator;
use rand::distributions::Alphanumeric;
use rand::prelude::*;
use reqwest;
use ring::digest::{Context, SHA512};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::webdriver::Session;

const CODEFORCES_RUST_LANG_ID: u32 = 75;

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
    content: Option<String>,
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

            let f = c.form(Locator::Css("#enterForm")).await?;
            f.set_by_name("handleOrEmail", &env::var("CODEFORCES_EMAIL").unwrap())
                .await?;
            f.set_by_name("password", &env::var("CODEFORCES_PASSWORD").unwrap())
                .await?;
            f.submit().await?;

            c.wait()
                .for_url(Url::parse("https://codeforces.com").unwrap())
                .await?
        }

        Ok(())
    }

    pub async fn submit(&mut self, problem: String) -> Result<(), fantoccini::error::CmdError> {
        if let Some(ref mut c) = self.client {
            c.goto("https://codeforces.com/problemset/submit").await?;

            let f = c.form(Locator::Css(".submit-form")).await?;
            f.set_by_name("submittedProblemCode", &problem.to_uppercase())
                .await?;

            let lang = c.find(Locator::Css("select[name='programTypeId']")).await?;
            lang.select_by_value(&CODEFORCES_RUST_LANG_ID.to_string())
                .await?;

            let filename = format!("src/bin/{}.rs", problem);
            let code = fs::read_to_string(filename).expect("failed to read from file");
            let code_area = c.find(Locator::Css(".ace_text-input")).await?;
            code_area.send_keys(&code).await?;

            f.submit().await?;

            c.wait()
                .for_url(Url::parse("https://codeforces.com/problemset/status").unwrap())
                .await?
        }
        Ok(())
    }

    pub async fn exit(&mut self) -> Result<(), fantoccini::error::CmdError> {
        if let Some(c) = self.client.take() {
            c.close().await?
        }
        Ok(())
    }
}

pub async fn get_problemset_problems(
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
        client.get(query).form(&params).send().await?.json().await?;

    println!("{:?}", resp);
    Ok(())
}

pub async fn get_user_info(handles: &str) -> Result<(), Box<dyn Error>> {
    let resp: CodeforcesResponse<Vec<User>> = reqwest::get(&format!(
        "https://codeforces.com/api/user.info?handles={}",
        handles
    ))
    .await?
    .json()
    .await?;
    println!("{:?}", resp);
    Ok(())
}

pub async fn get_blog_entry(number: u32) -> Result<(), Box<dyn Error>> {
    let resp: CodeforcesResponse<BlogEntry> = reqwest::get(&format!(
        "https://codeforces.com/api/blogEntry.view?blogEntryId={}",
        number
    ))
    .await?
    .json()
    .await?;
    println!("{:?}", resp);
    Ok(())
}

pub fn sign(method: String, params: HashMap<String, String>) -> String {
    let cf_key = env::var("CODEFORCES_API_KEY").unwrap();
    let cf_secret = env::var("CODEFORCES_API_SECRET").unwrap();
    let rng = rand::thread_rng();
    let rand_string: String =
        String::from_utf8(rng.sample_iter(Alphanumeric).take(6).collect::<Vec<_>>()).unwrap();
    let current_time = time::SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .expect("failed to get system time");
    let mut sorted_params = BTreeMap::new();
    for (param, value) in params {
        sorted_params.insert(param.to_string(), value.to_string());
    }
    sorted_params.insert("apiKey".to_string(), cf_key);
    sorted_params.insert("time".to_string(), format!("{:?}", current_time.as_secs()));
    let mut param_list = Vec::new();
    for (param, value) in sorted_params {
        param_list.push(format!("{}={}", param, value));
    }
    let param_string = param_list.join("&");
    let raw = format!(
        "{}/{}?{}#{}",
        &rand_string, &method, &param_string, &cf_secret
    );

    let mut context = Context::new(&SHA512);
    context.update(raw.as_bytes());
    let hash = HEXLOWER.encode(context.finish().as_ref());
    format!(
        "https://codeforces.com/api/{}?{}&apiSig={}{}",
        &method, &param_string, &rand_string, &hash
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use dotenv::dotenv;

    #[test]
    fn test_sign() {
        dotenv().ok();
        let mut params = HashMap::new();
        params.insert("contestId".to_string(), "566".to_string());
        let digest = sign("contest.hacks".to_string(), params);
        println!("{:?}", digest);
    }
}
