use vnc::client;
use vnc::client::{AuthChoice, AuthMethod};
use crate::config::Connection;

pub fn authenticate(con: &Connection, methods: &[AuthMethod]) -> Option<AuthChoice> {
    debug!("available authentication methods: {:?}", methods);
    for method in methods {
        match method {
            client::AuthMethod::None => return Some(client::AuthChoice::None),
            client::AuthMethod::Password => {
                return match con.password {
                    None => !panic!("VNC Auth not possible, due to missing 'password' arg"),
                    Some(ref password) => {
                        let mut key = [0; 8];
                        for (i, byte) in password.bytes().enumerate() {
                            if i == 8 {
                                break;
                            }
                            key[i] = byte
                        }
                        Some(client::AuthChoice::Password(key))
                    }
                }
            }
            client::AuthMethod::AppleRemoteDesktop => match (con.username, con.password) {
                (Some(username), Some(password)) => {
                    return Some(client::AuthChoice::AppleRemoteDesktop(
                        username.to_owned(),
                        password.to_owned(),
                    ))
                }
                _ => (),
            },
            _ => (),
        }
    }
    None
}
