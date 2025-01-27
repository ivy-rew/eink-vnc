use crate::vnc::auth;
use vnc::{Client, Encoding, Rect};

pub fn connect(con: Connection) -> Client {
    info!("connecting to {}:{}", con.host, con.port);
    let stream = match std::net::TcpStream::connect((con.host, con.port)) {
        Ok(stream) => stream,
        Err(error) => {
            error!("cannot connect to {}:{}: {}", con.host, con.port, error);
            std::process::exit(1)
        }
    };

    let mut vnc = match Client::from_tcp_stream(stream, !con.exclusive, |methods| auth::authenticate(&con, methods)) {
        Ok(vnc) => vnc,
        Err(error) => {
            error!("cannot initialize VNC session: {}", error);
            std::process::exit(1)
        }
    };

    let (width, height) = vnc.size();
    info!(
        "connected to \"{}\", {}x{} framebuffer",
        vnc.name(),
        width,
        height
    );

    let vnc_format = vnc.format();
    info!("received {:?}", vnc_format);

    vnc.set_encodings(&[Encoding::CopyRect, Encoding::Zrle])
        .unwrap();

    vnc.request_update(full_rect(vnc.size()), false)
        .unwrap();

    vnc
}

fn full_rect(size: (u16,u16)) -> Rect {
    Rect {
        left: 0,
        top: 0,
        width: size.0,
        height: size.1,
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Connection<'a> {
    pub host: &'a str,
    pub port: u16,
    pub username: Option<&'a str>,
    pub password: Option<&'a str>,
    pub exclusive: bool,
}
