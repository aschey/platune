use rcgen::{
    BasicConstraints, Certificate, CertificateParams, CertificateSigningRequest, DistinguishedName,
    DnType, ExtendedKeyUsagePurpose, IsCa, KeyUsagePurpose, RcgenError, SanType,
};
use tonic::transport::{ClientTlsConfig, Identity, ServerTlsConfig};
use uuid::Uuid;

fn test() -> Result<(), Box<dyn std::error::Error>> {
    let server_ca = gen_cert_for_ca()?;
    let client_ca = gen_cert_for_ca()?;

    let host_cert = gen_cert_for_server(&server_ca, "test".to_owned())?;
    let client_cert = gen_cert_for_server(&client_ca, "test".to_owned())?;
    let client_ca_pem = tonic::transport::Certificate::from_pem(client_ca.serialize_pem()?);

    let server_identity =
        Identity::from_pem(host_cert.signed_certificate_pem, host_cert.private_key_pem);
    let client_identity = Identity::from_pem(
        client_cert.signed_certificate_pem,
        client_cert.private_key_pem,
    );

    let ca = tonic::transport::Certificate::from_pem(server_ca.serialize_pem().unwrap());
    let client_tls = ClientTlsConfig::new()
        .ca_certificate(ca)
        .identity(client_identity)
        .domain_name("test");
    let server_tls = ServerTlsConfig::new()
        .identity(server_identity)
        .client_ca_root(client_ca_pem);
    Ok(())
}

fn gen_cert_for_ca() -> Result<Certificate, RcgenError> {
    let mut dn = DistinguishedName::new();
    dn.push(DnType::CountryName, "USA");
    dn.push(DnType::CommonName, "Auto-Generated CA");

    let mut params = CertificateParams::default();

    params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    params.alg = &rcgen::PKCS_ECDSA_P256_SHA256;
    params.distinguished_name = dn;

    params.key_usages = vec![KeyUsagePurpose::KeyCertSign, KeyUsagePurpose::CrlSign];

    Certificate::from_params(params)
}

struct ServerCertificate {
    private_key_pem: String,

    // Server certificate only; does not include complete certificate chain.
    signed_certificate_pem: String,
}

fn gen_cert_for_server(
    ca: &Certificate,
    dns_name: String,
) -> Result<ServerCertificate, RcgenError> {
    let mut dn = DistinguishedName::new();
    dn.push(DnType::OrganizationName, "Platune");
    dn.push(DnType::OrganizationalUnitName, "Platune Music Server");
    dn.push(DnType::CommonName, Uuid::new_v4().to_string());

    let mut params = CertificateParams::default();

    params.is_ca = IsCa::NoCa;
    params.alg = &rcgen::PKCS_ECDSA_P256_SHA256;
    params.distinguished_name = dn;
    params.subject_alt_names = vec![SanType::DnsName(dns_name)];
    params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];

    let unsigned = Certificate::from_params(params)?;

    let request_pem = unsigned.serialize_request_pem()?;

    let csr = CertificateSigningRequest::from_pem(&request_pem)?;

    let signed_pem = csr.serialize_pem_with_signer(ca)?;

    Ok(ServerCertificate {
        private_key_pem: unsigned.serialize_private_key_pem(),
        signed_certificate_pem: signed_pem,
    })
}
