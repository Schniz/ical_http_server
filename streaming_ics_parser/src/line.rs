use std::str::FromStr;

#[derive(Debug, PartialEq, Eq)]
pub struct Line {
    pub key: String,
    pub value: Option<String>,
}

impl FromStr for Line {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.splitn(2, ':');
        let key = parts.next().ok_or("missing key")?;
        let value = parts.next();
        Ok(Line {
            key: key.to_ascii_lowercase().to_string(),
            value: value.map(|s| s.to_string()),
        })
    }
}
