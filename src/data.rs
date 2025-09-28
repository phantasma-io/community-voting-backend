use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::config::ApiConfig;

pub struct ApiData {
    pub data_path: String,
    pub candidates: Vec<Candidate>,
    pub categories: Vec<Category>,
}

impl ApiData {
    const VOTES_DIR_NAME: &'static str = "votes";

    #[must_use]
    pub fn init(config: &ApiConfig) -> Self {
        Self {
            data_path: config.data_path.clone(),
            candidates: read_candidates(&config.data_path),
            categories: read_categories(&config.data_path)
        }
    }

    pub fn votes_path(&self) -> String {
        format!("{}/{}", self.data_path, Self::VOTES_DIR_NAME)
    }

    pub fn make_addr_vote_path(&self, addr: &str, category: &str) -> String {
        format!("{}/{}-{}.json", self.votes_path(), addr, category)
    }

    pub fn vote_exist(&self, vote: &Vote) -> bool {
        std::fs::exists(self.make_addr_vote_path(&vote.addr, &vote.category_slug)).unwrap_or(false)
    }

    pub fn get_addr_votes(&self, addr: &str) -> Vec<Vote> {
        let mut votes: Vec<Vote> = vec![];

        for entry_res in std::fs::read_dir(self.votes_path()).unwrap() {
            if let Ok(entry) = entry_res {
                if entry.file_name().as_os_str().to_str().unwrap_or_default().starts_with(&addr) {
                    let Ok(vote_raw) = std::fs::read_to_string(entry.path()) else {
                        continue;
                    };
                    let Ok(vote) = serde_json::from_str::<Vote>(&vote_raw) else {
                        continue;
                    };
                    votes.push(vote)
                }
            }
        }

        votes
    }

    pub fn persist_vote(&self, vote: Vote) -> Result<(), anyhow::Error> {
        let path = self.make_addr_vote_path(&vote.addr, &vote.category_slug);

        std::fs::create_dir_all(self.votes_path())?;

        let vote_raw = serde_json::to_string_pretty(&vote)?;

        std::fs::write(path, &vote_raw)?;

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Category {
    pub slug: String,
    pub name: String,
    #[serde(default)]
    pub extra: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Candidate {
    pub slug: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub img_url: Option<String>,
    #[serde(default)]
    pub extra: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Vote {
    #[serde(default="now_ms")]
    pub time_ms: u128,

    pub addr: String,
    // base16 string
    pub msg: String,
    pub signature: String,
    // random is base16 string
    #[serde(default)]
    pub random: String,
    #[serde(default = "default_sig_format")]
    pub sig_format: String,
    pub candidate_slug: String,
    pub category_slug: String,
    #[serde(default)]
    pub extra: Option<serde_json::Value>,
}

pub fn default_sig_format() -> String {
    "plain".to_string()
}

pub fn read_candidates(data_path: &str) -> Vec<Candidate> {
    let candidates_path = format!("{data_path}/candidates.json");
    let raw_data = std::fs::read(&candidates_path)
        .unwrap_or_else(|err| panic!("FAIL read candidates at {candidates_path}: {err}"));
    let data =
        serde_json::from_slice::<Vec<Candidate>>(&raw_data).expect("FAIL deserialize candidates");
    data
}

pub fn read_categories(data_path: &str) -> Vec<Category> {
    let path = format!("{data_path}/categories.json");
    let raw_data = std::fs::read(&path)
        .unwrap_or_else(|err| panic!("FAIL read categories at {path}: {err}"));
    let data =
        serde_json::from_slice::<Vec<Category>>(&raw_data).expect("FAIL deserialize categories");
    data
}

pub async fn verify_vote(explorer_api_url: &str, vote: &Vote) -> Result<bool, anyhow::Error> {
    let mut msg = vote.msg.clone();
    let sig = &vote.signature;
    let sig_format = &vote.sig_format;
    let addr = &vote.addr;

    if !vote.random.is_empty() {
        msg = format!("{}{}", vote.random, msg);
    }

    let url = format!(
        "{explorer_api_url}/api/v1/verifyMessage?message={}&signerAddress={}&signature={}&signatureFormat={sig_format}&messageFormat=Base16",
        msg, addr, sig
    );

    let res = reqwest::get(url)
        .await
        .map_err(|err| anyhow!("FAIL verify sig: {err}"))?
        .text()
        .await
        .map_err(|err| anyhow!("FAIL verify sig (deserialize): {err}"))?;

    match res.parse::<bool>() {
        Ok(value) => Ok(value),
        Err(_) => Err(anyhow!("FAIL verify sig, result: {res}")),
    }
}

pub fn now_ms() -> u128 {
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
    since_the_epoch.as_millis()
}

