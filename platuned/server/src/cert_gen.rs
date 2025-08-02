use std::path::Path;
use std::{env, fs};

use rcgen::{
    BasicConstraints, Certificate, CertificateParams, DistinguishedName, DnType,
    ExtendedKeyUsagePurpose, IsCa, Issuer, KeyPair, KeyUsagePurpose,
};
use time::OffsetDateTime;
use tonic::transport::{Identity, ServerTlsConfig};
use tracing::info;
use uuid::Uuid;

pub(crate) struct TlsConfig {
    pub(crate) ca_pem: String,
    pub(crate) cert: ServerCertificate,
}

pub(crate) fn get_tonic_tls_config(
    server_tls: TlsConfig,
    client_tls: Option<TlsConfig>,
) -> ServerTlsConfig {
    let server_identity = Identity::from_pem(
        server_tls.cert.signed_certificate_pem,
        server_tls.cert.private_key_pem,
    );

    let server_tls_config = ServerTlsConfig::new().identity(server_identity);
    if let Some(client_tls) = client_tls {
        let client_ca_pem = tonic::transport::Certificate::from_pem(client_tls.ca_pem);
        server_tls_config.client_ca_root(client_ca_pem)
    } else {
        server_tls_config
    }
}

pub(crate) async fn get_tls_config(path: &Path) -> Result<TlsConfig, rcgen::Error> {
    if path.join("ca.pem").exists()
        && path.join("cert.pem").exists()
        && path.join("cert.key").exists()
    {
        return Ok(TlsConfig {
            ca_pem: fs::read_to_string(path.join("ca.pem")).unwrap(),
            cert: ServerCertificate {
                private_key_pem: fs::read_to_string(path.join("cert.key")).unwrap(),
                signed_certificate_pem: fs::read_to_string(path.join("cert.pem")).unwrap(),
            },
        });
    }

    info!("Generating TLS certs");
    let path = path.to_owned();
    let res = tokio::task::spawn_blocking(move || {
        let (ca, server_key_pair) = gen_cert_for_ca()?;

        let cert = gen_cert_for_server(&server_key_pair)?;

        fs::create_dir_all(&path).unwrap();
        fs::write(path.join("ca.pem"), ca.pem()).unwrap();
        fs::write(path.join("cert.pem"), &cert.signed_certificate_pem).unwrap();
        fs::write(path.join("cert.key"), &cert.private_key_pem).unwrap();
        Ok(TlsConfig {
            ca_pem: ca.pem(),
            cert,
        })
    });
    res.await.unwrap()
}

fn gen_cert_for_ca() -> Result<(Certificate, Issuer<'static, KeyPair>), rcgen::Error> {
    let mut params = CertificateParams::new(Vec::default()).unwrap();

    params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);

    params
        .distinguished_name
        .push(DnType::CommonName, "Platune");

    params.not_before = OffsetDateTime::now_utc()
        .checked_sub(time::Duration::days(365))
        .unwrap();
    params.not_after = OffsetDateTime::now_utc()
        .checked_add(time::Duration::days(365 * 10))
        .unwrap();
    params.key_usages.push(KeyUsagePurpose::KeyCertSign);
    params.key_usages.push(KeyUsagePurpose::CrlSign);
    params.key_usages.push(KeyUsagePurpose::DigitalSignature);

    let key_pair = KeyPair::generate()?;
    let cert = params.self_signed(&key_pair)?;

    Ok((cert, Issuer::new(params, key_pair)))
}

pub(crate) struct ServerCertificate {
    private_key_pem: String,

    // Server certificate only; does not include complete certificate chain.
    signed_certificate_pem: String,
}

fn gen_cert_for_server(
    issuer: &Issuer<'static, KeyPair>,
) -> Result<ServerCertificate, rcgen::Error> {
    let mut dn = DistinguishedName::new();
    dn.push(DnType::OrganizationName, "Platune");
    dn.push(DnType::OrganizationalUnitName, "Platune Music Server");
    dn.push(DnType::CommonName, Uuid::new_v4().to_string());

    let hosts: Vec<_> = env::var("PLATUNE_HOSTS")
        .unwrap()
        .split(',')
        .map(|s| s.to_string())
        .collect();
    let mut params = CertificateParams::new(hosts).unwrap();
    params.use_authority_key_identifier_extension = true;
    params.is_ca = IsCa::NoCa;

    params.distinguished_name = dn;

    params.not_before = OffsetDateTime::now_utc()
        .checked_sub(time::Duration::days(1))
        .unwrap();
    params.not_after = OffsetDateTime::now_utc()
        .checked_add(time::Duration::days(365 * 10))
        .unwrap();
    params
        .extended_key_usages
        .push(ExtendedKeyUsagePurpose::ServerAuth);
    params.key_usages.push(KeyUsagePurpose::DigitalSignature);

    let key_pair = KeyPair::generate()?;
    let signed_pem = params.signed_by(&key_pair, issuer)?;

    Ok(ServerCertificate {
        private_key_pem: key_pair.serialize_pem(),
        signed_certificate_pem: signed_pem.pem(),
    })
}
