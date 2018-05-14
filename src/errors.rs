error_chain! {
    foreign_links {
        Chrono(::chrono::ParseError);
//        Failure(::failure::Error);
        Io(::std::io::Error);
        Pie(::std::num::ParseIntError);
        Reqwest(::reqwest::Error);
        Toml(::toml::de::Error);
    }
}
