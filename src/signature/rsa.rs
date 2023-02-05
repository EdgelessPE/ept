use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};
use openssl::hash::MessageDigest;
use openssl::pkey::PKey;
use openssl::sign::{Signer, Verifier};

pub fn sign_with_rsa(private_key: String, digest: String) -> Result<String> {
    let private = PKey::private_key_from_pem(private_key.as_bytes())?;
    let mut signer = Signer::new(MessageDigest::sha256(), &private)?;
    signer.update(digest.as_bytes())?;
    let signature = signer.sign_to_vec()?;
    let signature_base64 = general_purpose::STANDARD.encode(&signature);

    Ok(signature_base64)
}

pub fn verify_with_rsa(public_key: String, digest: String, signature: String) -> Result<bool> {
    let public = PKey::public_key_from_pem(public_key.as_bytes())?;
    let mut verifier = Verifier::new(MessageDigest::sha256(), &public).unwrap();
    verifier.update(digest.as_bytes())?;
    let signature_decoded = general_purpose::STANDARD.decode(&signature)?;
    let v_res = verifier.verify(&signature_decoded)?;
    Ok(v_res)
}

#[test]
fn test_sign_with_rsa() {
    let private_key = "-----BEGIN RSA PRIVATE KEY-----
MIIEpAIBAAKCAQEAkpG8ZaqMf55BxmGFzZ2g7e1S6hr5Ns275N8Usi8OtpvbqR5U9Gp4+DYy
CLgS1IX23LFFFSNjfYpKXyVr885YVdZOJjuhtxdXwzLtQ4vd9g8g1lJKwF9yIoGgsZyaX9lC
9cjHezntBOKumgQWsMdzCbXK2j6DkG7yB3NTSVd63PH/HFikVhiW05F3E963B94h0XyY5nfC
m+cttV5BBDRDdpCxfCqpbx5aj24FzyqM4WHDzLVhKuXKBEB5pfD9VZhlxdZn9SnsRLli6FO2
gNd/yhEPJ6nFL7SLBzM5M8nnoVokWwjL+p54zf7hu2EuNyJjgb9ZqfwNUXILc1LSEGE7qQID
AQABAoIBACh3UBqJoczCNsq8tiJ0uK+37EJyPAgjeRLRfHdNgrRsB5ODqlTo6Iku/VVm7Nv8
OJHp53bUlG1etvXZ8RoZCE56oozvvdA9A6AC+XrCrP94YcqKYdUHBQ392A3xfLWl2FTfoCOn
dIb6xtYC9vjLuDkgFed3hv9jgjMIZiBDpMpHIDWMu8cwF0DN0jaSFdsP4Bp4hRvv4c+cEt3E
1PhBmcH5+O7B35QMjlsLCmPLh+mganjG0pxmNDoPE0t93/SO7XpoGiDLGiUpJaV+UYcQqDFN
sxpHmBoRbjxbK6IdAHlfQ5ruIqAX2NSIFij5yM+yN3y7KQW/qJ0k7Jy9Ja+zLAECgYEA80uZ
kIsDGYt0Fp7PwhitbKaPwWoZBV8Gf2i2n7jIagoXNhRMpuMZ511wqGSOTdnoJ1KMPLxXYdtZ
LEb+MJZr4BnbtjiDuYPQiXSwqDnP5IhCUGktFUiAxAEuPi8jj08TlYWTm1C8NprNAMDV+L9C
40X+OLYzTRNaWFcMiEcSkEECgYEAmjkXWBpCiAzRagQYCLKN5oMd+IBnCKCYxbP4YNkCgCdZ
fnOSFdFtXkop4jNu1Y0WWFmoZq91Eo8B/sE0y6MXU9KNt1rzogpn8lf92GjI99gmO4+eL1sr
K8M/ac9NM8G9ax7kLQnUxrx6MgFdM5Ro683QC2fY9bY/LUzdBMI/0WkCgYEAoNgnIw80Mmwm
iFnf4mMsLDuFqIn7Frj2876Hldq07J3VMJSFBIP5eSMmOr2X8tIQEAcb3X9qibBZKNOacwbP
NK4DullshHYnpOg3blAiJ+UJal3OR1bSgkKBjuzdJn5R5TUVG8ZpV/RJeakDDNttXhHE+ztB
eUBFJ0gNaf8Kv8ECgYBqgbsJcTk5VADbwnAGsakl5K8yCxsc7iwTfTKvT69WadZ4acAdqUBq
ubUrLnIAsSsZYPHX2Jx1cKXkFfIsIDnf/a05T2qqIZ2f0/zPE66W24Or5odMFR4/XtvQawXa
FJaIABF8uSllBo0tM5v2HyxGjSB8f/9p0a7XzhllS/Fe4QKBgQCImJkio+eRzUCYWHWkT0Jo
VdaFLgwThYxDeTOWBzPM/ejx7mGTtckSEcPjcooF3VATSmjxUqRUXbYFWvJT8WV9UE1cQmCD
cP5wONFiB16hX1KCrcMzxWwkBOHHJv/jjBoSDRKSdne6f8M66Kv5s/wXtUK8Gv13wtaSwYZg
ktIqgQ==
-----END RSA PRIVATE KEY-----
";
    let public_key = "-----BEGIN RSA PUBLIC KEY-----
MIIBCgKCAQEAkpG8ZaqMf55BxmGFzZ2g7e1S6hr5Ns275N8Usi8OtpvbqR5U9Gp4+DYyCLgS
1IX23LFFFSNjfYpKXyVr885YVdZOJjuhtxdXwzLtQ4vd9g8g1lJKwF9yIoGgsZyaX9lC9cjH
ezntBOKumgQWsMdzCbXK2j6DkG7yB3NTSVd63PH/HFikVhiW05F3E963B94h0XyY5nfCm+ct
tV5BBDRDdpCxfCqpbx5aj24FzyqM4WHDzLVhKuXKBEB5pfD9VZhlxdZn9SnsRLli6FO2gNd/
yhEPJ6nFL7SLBzM5M8nnoVokWwjL+p54zf7hu2EuNyJjgb9ZqfwNUXILc1LSEGE7qQIDAQAB
-----END RSA PUBLIC KEY-----
";
    let signature = sign_with_rsa(private_key.to_string(), "digest".to_string()).unwrap();
    assert_eq!(signature,String::from("Jhr/pWdzd5jze829bsWF1lUDd8baA5WszKOFjqhq2mT+dkJ7e+k3oV6v/Zx4AtcJ5eorXAfJaSvjIZ65ZmAo3fIcL0+NWLAAGVu3x13lmp9MiUOKpybEqAEkdFXaaMQZjsDvTSMxGt+4PVWDfP0wvwYPCsoKlQf17LPUPLVlgMhtpiA3XO12n0M7TOZGAehg1JrL1zxiYvgBsllbWRtbsU/Dzef34jx1Qx1gfThM+t6eIEEzWnGcnxMk9EilxVAlkgy0qv2GNWTEHb52B8BHcxW9ZkJ2y+u5bHCO6pSTFtv1i1nZPYbHYgSuJ3YYkof+YlsWqNAGcJDUconqVvZU3g=="));
    let validation =
        verify_with_rsa(public_key.to_string(), "digest".to_string(), signature).unwrap();
    println!("{}", validation);
    assert!(validation);
}
