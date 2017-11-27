#[cfg(feature = "postgres")]
mod postgres;
#[cfg(feature = "rusqlite")]
mod rusqlite;

use regex::Regex;

#[cfg(feature = "postgres")]
pub use self::postgres::*;
#[cfg(feature = "rusqlite")]
pub use self::rusqlite::*;

pub fn to_table_name<S: AsRef<str>>(struct_name: S) -> String {
    let re = Regex::new(r"(?P<l>[[:upper:]])").unwrap();

    re.replace_all(struct_name.as_ref(), "_$l")[1..].to_lowercase()
}
