use std::{env, fs, thread, time};

use fantoccini::{Client, Locator};

#[derive(Clone)]
pub struct CF {
  client: Option<Client>,
}

impl CF {
  pub async fn login(&mut self) -> Result<(), fantoccini::error::CmdError> {
    self.client = Some(
      Client::new("http://localhost:4444")
        .await
        .expect("failed to connect to WebDriver"),
    );

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

pub fn create_client() -> CF {
  CF { client: None }
}
