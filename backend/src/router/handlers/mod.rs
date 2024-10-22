use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Html,
};

use crate::state::AppState;

pub mod auth;
pub mod invoices;
pub mod lnurlp;

#[axum_macros::debug_handler]
pub async fn handle_home(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Html<String>, (StatusCode, String)> {
    let replit_user_id = headers
        .get("X-Replit-User-Id")
        .and_then(|h| h.to_str().ok());
    let replit_user_name = headers
        .get("X-Replit-User-Name")
        .and_then(|h| h.to_str().ok());
    let replit_profile_pic = headers
        .get("X-Replit-Profile-Pic")
        .and_then(|h| h.to_str().ok());

    match (replit_user_id, replit_user_name, replit_profile_pic) {
        (Some(user_id), Some(user_name), Some(profile_pic)) => {
            // User is logged in
            let user = match state.db.users().get_by_name(user_name).await {
                Ok(Some(user)) => user,
                Ok(None) => return Ok(generate_registration_html(user_id, user_name, profile_pic)),
                Err(e) => {
                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Error getting user: {}", e),
                    ))
                }
            };

            Ok(generate_html_response(user_id, user_name, &user.pubkey))
        }
        _ => {
            // User is not logged in
            Ok(generate_login_html())
        }
    }
}

fn generate_html_response(user_id: &str, user_name: &str, token: &str) -> Html<String> {
    Html(format!(
        r#"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>Replex-Auth</title>
            <style>
                body {{ font-family: Arial, sans-serif; margin: 20px; }}
                h1 {{ color: #333; }}
                p {{ color: #555; }}
            </style>
        </head>
        <body>
            <h1>Replex-Auth</h1>
            <p><strong>User ID:</strong> {}</p>
            <p><strong>User Name:</strong> {}</p>
            <p><strong>Connection Code:</strong> {}</p>
            <div>
                <script
                    authed="location.reload()"
                    src="https://auth.util.repl.co/script.js"
                ></script>
            </div>
        </body>
        </html>
        "#,
        user_id, user_name, token
    ))
}

fn generate_registration_html(user_id: &str, user_name: &str, profile_pic: &str) -> Html<String> {
    Html(format!(
        r#"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>Replex-Auth Registration</title>
            <style>
                body {{ font-family: Arial, sans-serif; margin: 20px; }}
                h1 {{ color: #333; }}
                p {{ color: #555; }}
                form {{ margin-top: 20px; }}
                input[type="text"] {{ width: 300px; padding: 5px; }}
                input[type="submit"] {{ margin-top: 10px; padding: 5px 10px; }}
            </style>
        </head>
        <body>
            <h1>Replex-Auth Registration</h1>
            <p><strong>User ID:</strong> {}</p>
            <p><strong>User Name:</strong> {}</p>
            <img src="{}" alt="Profile Picture" style="width: 100px; height: 100px;">
            <form action="/register" method="POST">
                <input type="hidden" name="user_id" value="{}">
                <input type="hidden" name="user_name" value="{}">
                <input type="hidden" name="profile_pic" value="{}">
                <label for="pubkey">Public Key:</label><br>
                <input type="text" id="pubkey" name="pubkey" required><br>
                <input type="submit" value="Register">
            </form>
            <div>
                <script
                    authed="location.reload()"
                    src="https://auth.util.repl.co/script.js"
                ></script>
            </div>
        </body>
        </html>
        "#,
        user_id, user_name, profile_pic, user_id, user_name, profile_pic
    ))
}

fn generate_login_html() -> Html<String> {
    Html(format!(
        r#"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>Replex-Auth Login</title>
            <style>
                body {{ font-family: Arial, sans-serif; margin: 20px; }}
                h1 {{ color: #333; }}
                button {{ padding: 10px 20px; font-size: 16px; }}
            </style>
        </head>
        <body>
            <h1>Replex-Auth Login</h1>
            <button onclick="LoginWithReplit()">Login with Replit</button>
            <script>
                function LoginWithReplit() {{
                    window.addEventListener("message", authComplete);
                    var h = 500;
                    var w = 350;
                    var left = screen.width / 2 - w / 2;
                    var top = screen.height / 2 - h / 2;

                    var authWindow = window.open(
                        "https://replit.com/auth_with_repl_site?domain=" + location.host,
                        "_blank",
                        "modal =yes, toolbar=no, location=no, directories=no, status=no, menubar=no, scrollbars=no, resizable=no, copyhistory=no, width=" +
                        w +
                        ", height=" +
                        h +
                        ", top=" +
                        top +
                        ", left=" +
                        left
                    );

                    function authComplete(e) {{
                        if (e.data !== "auth_complete") {{
                            return;
                        }}

                        window.removeEventListener("message", authComplete);

                        authWindow.close();
                        location.reload();
                    }}
                }}
            </script>
        </body>
        </html>
        "#
    ))
}
