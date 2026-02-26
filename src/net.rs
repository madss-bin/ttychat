use anyhow::{bail, Context, Result};
use rustls::pki_types::ServerName;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_rustls::TlsConnector;

pub mod net_types {
    use anyhow::Result;
    pub type SignatureFn = Box<dyn Fn(&str) -> Result<String> + Send>;
}

#[derive(Debug, Clone, Deserialize)]
pub struct Challenge {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub nonce: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthMsg {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub pubkey: String,
    pub username: String,
    pub sig: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct EnrollMsg {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub username: String,
    pub pubkey: String,
    pub invite_code: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthResponse {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ClientMsg {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub text: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ServerMsg {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub from: Option<String>,
    pub text: Option<String>,
    #[serde(alias = "time", alias = "ts", alias = "created", alias = "created_at", alias = "datetime", alias = "date")]
    pub timestamp: Option<serde_json::Value>,
    pub users: Option<serde_json::Value>,
    pub online: Option<Vec<String>>,
    pub names: Option<Vec<String>>,
    pub list: Option<Vec<String>>,
    pub user_list: Option<Vec<String>>,
    pub action: Option<String>,
    pub data: Option<String>,
}

#[derive(Debug, Clone)]
pub enum NetEvent {
    Connected,
    AuthOk {
        username: String,
    },
    AuthFail {
        reason: String,
    },
    Message(Box<ServerMsg>),
    AdminResponse {
        action: String,
        data: String,
    },
    Error(String),
    Disconnected,
}

#[derive(Debug)]
pub enum NetCommand {
    SendMessage(String),
    SendAdminCmd(String),
}

fn build_tls_config(insecure: bool) -> Result<Arc<rustls::ClientConfig>> {
    if insecure {
        let config = rustls::ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(NoCertVerifier))
            .with_no_client_auth();
        return Ok(Arc::new(config));
    }

    let mut root_store = rustls::RootCertStore::empty();
    root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

    let native = rustls_native_certs::load_native_certs();
    for cert in native.certs {
        let _ = root_store.add(cert);
    }

    let config = rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();
    Ok(Arc::new(config))
}

#[derive(Debug)]
struct NoCertVerifier;

impl rustls::client::danger::ServerCertVerifier for NoCertVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::pki_types::CertificateDer<'_>,
        _intermediates: &[rustls::pki_types::CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dsa: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dsa: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        vec![
            rustls::SignatureScheme::RSA_PKCS1_SHA256,
            rustls::SignatureScheme::RSA_PKCS1_SHA384,
            rustls::SignatureScheme::RSA_PKCS1_SHA512,
            rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
            rustls::SignatureScheme::ECDSA_NISTP384_SHA384,
            rustls::SignatureScheme::RSA_PSS_SHA256,
            rustls::SignatureScheme::RSA_PSS_SHA384,
            rustls::SignatureScheme::RSA_PSS_SHA512,
            rustls::SignatureScheme::ED25519,
        ]
    }
}

pub struct NetParams {
    pub server: String,
    pub username: String,
    pub pubkey_b64: String,
    pub sig_fn: net_types::SignatureFn,
    pub enroll_code: Option<String>,
    pub insecure: bool,
}

pub async fn connect(
    params: NetParams,
    event_tx: mpsc::UnboundedSender<NetEvent>,
    mut cmd_rx: mpsc::UnboundedReceiver<NetCommand>,
) {
    tokio::spawn(async move {
        if let Err(e) = run_session(params, event_tx.clone(), &mut cmd_rx).await {
            let _ = event_tx.send(NetEvent::Error(format!("{e:#}")));
        }
        let _ = event_tx.send(NetEvent::Disconnected);
    });
}

async fn run_session(
    params: NetParams,
    event_tx: mpsc::UnboundedSender<NetEvent>,
    cmd_rx: &mut mpsc::UnboundedReceiver<NetCommand>,
) -> Result<()> {
    let host = params
        .server
        .split(':')
        .next()
        .context("Invalid server address")?
        .to_string();

    let tls_config = build_tls_config(params.insecure)?;
    let connector = TlsConnector::from(tls_config);
    let server_name = ServerName::try_from(host.clone())
        .map_err(|_| anyhow::anyhow!("Invalid hostname '{}' — use host:port format", host))?;

    let tcp = TcpStream::connect(&params.server)
        .await
        .map_err(|e| anyhow::anyhow!("Cannot connect to '{}': {}", params.server, e))?;

    let stream = connector
        .connect(server_name, tcp)
        .await
        .map_err(|e| anyhow::anyhow!("TLS handshake failed (try --insecure?): {}", e))?;

    let _ = event_tx.send(NetEvent::Connected);

    let (reader_half, mut writer) = tokio::io::split(stream);
    let mut reader = BufReader::new(reader_half);
    let mut line = String::new();

    line.clear();
    reader.read_line(&mut line).await?;
    let challenge: Challenge = serde_json::from_str(line.trim())
        .context("Expected challenge from server")?;

    if challenge.msg_type != "challenge" {
        bail!("Expected 'challenge', got '{}'", challenge.msg_type);
    }

    let username = params.username.clone();
    if let Some(invite_code) = &params.enroll_code {
        let msg = EnrollMsg {
            msg_type: "enroll".into(),
            username: params.username.clone(),
            pubkey: params.pubkey_b64.clone(),
            invite_code: invite_code.clone(),
        };
        let mut out = serde_json::to_vec(&msg)?;
        out.push(b'\n');
        writer.write_all(&out).await?;
    } else {
        let sig = (params.sig_fn)(&challenge.nonce)?;
        let auth = AuthMsg {
            msg_type: "auth".into(),
            pubkey: params.pubkey_b64.clone(),
            username: params.username.clone(),
            sig,
        };
        let mut out = serde_json::to_vec(&auth)?;
        out.push(b'\n');
        writer.write_all(&out).await?;
    }

    line.clear();
    reader.read_line(&mut line).await?;
    let resp: AuthResponse = serde_json::from_str(line.trim())
        .context("Expected auth response")?;

    if resp.msg_type == "auth_ok" {
        let _ = event_tx.send(NetEvent::AuthOk { username: username.clone() });
    } else {
        let reason = resp.reason.unwrap_or_else(|| "auth_fail".into());
        let _ = event_tx.send(NetEvent::AuthFail { reason });
        return Ok(());
    }

    loop {
        line.clear();
        tokio::select! {
            result = reader.read_line(&mut line) => {
                let n = result?;
                if n == 0 { break; }
                let trimmed = line.trim();
                if trimmed.is_empty() { continue; }

                let server_msg: ServerMsg = match serde_json::from_str(trimmed) {
                    Ok(m) => m,
                    Err(_) => continue,
                };

                match server_msg.msg_type.as_str() {
                    "msg" | "presence" => { let _ = event_tx.send(NetEvent::Message(Box::new(server_msg))); }
                    "admin_res" => {
                        let _ = event_tx.send(NetEvent::AdminResponse {
                            action: server_msg.action.unwrap_or_default(),
                            data: server_msg.data.unwrap_or_default(),
                        });
                    }
                    _ => {}
                }
            }

            cmd = cmd_rx.recv() => {
                match cmd {
                    Some(NetCommand::SendMessage(text)) => {
                        let m = ClientMsg { msg_type: "msg".into(), text };
                        let mut out = serde_json::to_vec(&m)?;
                        out.push(b'\n');
                        writer.write_all(&out).await?;
                    }
                    Some(NetCommand::SendAdminCmd(action)) => {
                        let m = serde_json::json!({ "type": "admin_cmd", "action": action });
                        let mut out = serde_json::to_vec(&m)?;
                        out.push(b'\n');
                        writer.write_all(&out).await?;
                    }
                    None => break,
                }
            }
        }
    }

    Ok(())
}
