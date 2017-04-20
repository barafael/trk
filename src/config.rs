#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub show_commits : bool,
    pub repository   : Option<String>,
    pub user_name    : Option<String>,
}

impl Config {
    pub fn new() -> Config {
        Config {
            show_commits : true,
            repository   : None,
            user_name    : None,
        }
    }
}
