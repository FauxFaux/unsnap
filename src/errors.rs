error_chain! {
    foreign_links {
        Io(::std::io::Error);
        Pie(::std::num::ParseIntError);
        Toml(::toml::de::Error);
    }
}
