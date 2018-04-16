error_chain! {
    foreign_links {
        Io(::std::io::Error);
        Pie(::std::num::ParseIntError);
        Reqwest(::reqwest::Error);
        Toml(::toml::de::Error);
    }
}
