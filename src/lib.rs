use dotenv::dotenv;
use oauth2::basic::BasicClient;
use oauth2::reqwest::http_client;
use oauth2::{
    AccessToken, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl,
    RefreshToken, Scope, TokenResponse, TokenUrl,
};
use std::env;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;

#[derive(Debug)]
pub struct Intuit {
    pub state: String,
    pub access_token: AccessToken,
    pub refresh_token: RefreshToken,
    pub realmId: String,
}

impl Intuit {
    pub fn build() -> Intuit {
        let mut result = Intuit {
            access_token: AccessToken::new(String::from("")),
            refresh_token: RefreshToken::new(String::from("")),
            state: String::from(""),
            realmId: String::from(""),
        };

        dotenv().ok();

        let intuit_client_id =
            ClientId::new(env::var("CLIENT_ID").expect("Missing intuit Client_ID"));
        let intuit_client_secret =
            ClientSecret::new(env::var("CLIENT_SECRET").expect("Missing inuit Client_Secret"));
        let auth_url = AuthUrl::new("https://appcenter.intuit.com/connect/oauth2".to_string())
            .expect("No url");
        let token_url =
            TokenUrl::new("https://oauth.platform.intuit.com/oauth2/v1/tokens/bearer".to_string())
                .expect("Invalid token endpoint URL");

        let client = BasicClient::new(
            intuit_client_id,
            Some(intuit_client_secret),
            auth_url,
            Some(token_url),
        )
        .set_redirect_uri(
            RedirectUrl::new("http://localhost:12031/".to_string()).expect("Invalid redirect URL"),
        );

        let (authorize_url, csrf_state) = client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("openid".to_string()))
            .add_scope(Scope::new("com.intuit.quickbooks.accounting".to_string()))
            .url();

        println!(
            "Open this URL in your browser:\n{}\n",
            authorize_url.to_string()
        );

        let listener = TcpListener::bind("127.0.0.1:12031").unwrap();
        for stream in listener.incoming() {
            if let Ok(mut stream) = stream {
                let code;
                let state;
                let realmId: String;
                {
                    let mut reader = BufReader::new(&stream);

                    let mut request_line = String::new();
                    reader.read_line(&mut request_line).unwrap();

                    let redirect_url = request_line.split_whitespace().nth(1).unwrap();
                    println!("{}", redirect_url);

                    let url =
                        url::Url::parse(&("http://localhost".to_string() + redirect_url)).unwrap();

                    let code_pair = url
                        .query_pairs()
                        .find(|pair| {
                            let &(ref key, _) = pair;
                            key == "code"
                        })
                        .unwrap();

                    let pair2 = url
                        .query_pairs()
                        .find(|pair| {
                            let &(ref key, _) = pair;
                            key == "realmId"
                        })
                        .unwrap();

                    let (_, value) = code_pair;
                    code = AuthorizationCode::new(value.into_owned());
                    let (_, value2) = pair2;
                    result.realmId = value2.into_owned();

                    let state_pair = url
                        .query_pairs()
                        .find(|pair| {
                            let &(ref key, _) = pair;
                            key == "state"
                        })
                        .unwrap();

                    let (_, value) = state_pair;
                    state = CsrfToken::new(value.into_owned());
                }

                let message = "Go back to your terminal :)";
                let response = format!(
                    "HTTP/1.1 200 OK\r\ncontent-length: {}\r\n\r\n{}",
                    message.len(),
                    message
                );
                stream.write_all(response.as_bytes()).unwrap();

                println!("Intuit returned the following code:\n{}\n", code.secret());
                println!(
                    "Intuit returned the following state:\n{} (expected `{}`)\n",
                    state.secret(),
                    csrf_state.secret()
                );

                result.state = String::from(state.secret());

                // Exchange the code with a token.
                let token_res = client.exchange_code(code).request(http_client);

                println!("Intuit returned the following token:\n{:?}\n", token_res);

                result.access_token = token_res.as_ref().unwrap().access_token().clone();
                result.refresh_token = token_res.unwrap().refresh_token().unwrap().clone();

                // The server will terminate itself after collecting the first code.

                break;
            }
        }
        result
    }
}
