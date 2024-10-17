use std::fs::read_to_string;

pub mod lnurlp;
pub mod register;

#[axum_macros::debug_handler]
pub async fn handle_readme() -> String {
    read_to_string("README.md").expect("Could not read README.md")
}
