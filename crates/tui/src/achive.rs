use std::{collections::HashMap, path::PathBuf, str::FromStr};

use anyhow::{anyhow, Context, Result};

use crate::difficulty::DifficultyKind;

const FILE: &str = "achivements.csv";
const SEP: &str = ",";

pub type AchivementMap = HashMap<String, Vec<Achivement>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Achivement {
    pub username: String,
    pub difficulty: DifficultyKind,
    pub score: usize,
}

impl PartialOrd for Achivement {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Achivement {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (&self.username, self.difficulty).cmp(&(&other.username, other.difficulty))
    }
}

pub fn save_achivement(achivement: Achivement) -> Result<()> {
    let mut achivements = read_achivements()?;
    if let Some(found) = achivements
        .iter_mut()
        .find(|e| e.username == achivement.username && e.difficulty == achivement.difficulty)
    {
        if found.score < achivement.score {
            *found = achivement;
        }
    } else {
        achivements.push(achivement);
    }
    achivements.sort_unstable();

    let res = achivements
        .into_iter()
        .map(|a| {
            if a.username.contains(SEP) {
                return Err(anyhow!("username cannot contain {SEP}"));
            }
            Ok([
                a.username,
                a.difficulty.to_string().to_lowercase(),
                a.score.to_string(),
            ]
            .join(SEP)
                + "\n")
        })
        .collect::<Result<String>>()?;

    let res = achivements_header() + "\n" + res.as_str();
    std::fs::write(achivements_file(), res).context("failed to write achivements")?;

    Ok(())
}

pub fn read_achivements() -> Result<Vec<Achivement>> {
    let file = achivements_file();
    if !file.exists() {
        std::fs::create_dir_all(config_dif()).context("failed to create config directory")?;
        std::fs::write(&file, achivements_header() + "\n")
            .context("failed to write achivements")?;
    }

    std::fs::read_to_string(file)
        .context("failed to read achivements")?
        .lines()
        .skip(1)
        .map(|l| l.split(SEP).map(|v| v.trim()).collect())
        .map(|l: Vec<_>| {
            let &[username, difficulty, score] = l.as_slice() else {
                return Err(anyhow!("unexpected elements count in entry"));
            };

            Ok(Achivement {
                username: username.to_owned(),
                difficulty: DifficultyKind::from_str(difficulty)
                    .map_err(|e| anyhow!("{e}"))
                    .context("invalid difficulty")?,
                score: str::parse(score).context("invalid score")?,
            })
        })
        .collect()
}

pub fn achivements2map(achivements: &[Achivement]) -> AchivementMap {
    let mut res = HashMap::new();
    for a in achivements {
        res.entry(a.username.clone())
            .and_modify(|e: &mut Vec<_>| e.push(a.clone()))
            .or_insert(vec![a.clone()]);
    }
    res
}

fn achivements_header() -> String {
    ["username", "difficulty", "score"].join(SEP)
}

fn achivements_file() -> PathBuf {
    config_dif().join(FILE)
}

fn config_dif() -> PathBuf {
    dirs::config_dir()
        .expect("config dir always exists")
        .join(crate::PKG_NAME)
}
