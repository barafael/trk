#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub show_commits : bool,
    pub repository   : String,
    pub user_name    : String,
}

impl Config {
    pub fn new() -> Config {
        Config {
            show_commits : true,
            repository   : String::new(),
            user_name    : String::new(),
        }
    }
}
