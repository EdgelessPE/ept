use anyhow::{anyhow, Result};
use base64::{engine::general_purpose, Engine as _};
use ed25519_compact::{Noise, PublicKey, SecretKey, Signature};

fn slice_to_array_64<T>(slice: &[T]) -> Result<&[T; 64]> {
    if slice.len() == 64 {
        let ptr = slice.as_ptr() as *const [T; 64];
        unsafe { Ok(&*ptr) }
    } else {
        Err(anyhow!("Error:Can't convert signature to 64 bytes array"))
    }
}

pub fn sign_with_ecdsa(private_key: &String, digest: &String) -> Result<String> {
    let private = SecretKey::from_pem(private_key)?;
    let signature = private.sign(digest.as_bytes(), Some(Noise::generate()));
    let signature_base64 = general_purpose::STANDARD.encode(signature);

    Ok(signature_base64)
}

pub fn verify_with_ecdsa(public_key: &String, digest: &String, signature: &String) -> Result<bool> {
    let public = PublicKey::from_pem(public_key)?;
    let signature_decoded = general_purpose::STANDARD.decode(signature)?;
    let arr = slice_to_array_64(&signature_decoded[..])?;
    let signature = Signature::new(*arr);
    let v_res = public.verify(digest.as_bytes(), &signature);
    Ok(v_res.is_ok())
}

#[test]
fn test_ecdsa() {
    let public_key = "-----BEGIN PUBLIC KEY-----
MCowBQYDK2VwAyEAhP93O5kkdSAnRy6ed8U+3Z0VcUKCzFz9m92E7jZGHUM=
-----END PUBLIC KEY-----"
        .to_string();
    let private_key = "-----BEGIN PRIVATE KEY-----
MC4CAQAwBQYDK2VwBCIEIBiVVXXhLTr/EY/FROnl67TVJz/jGV1WWN9HgptLMNWO
-----END PRIVATE KEY-----"
        .to_string();

    let signature = sign_with_ecdsa(&private_key, &"114514".to_string()).unwrap();
    println!("{signature}");
    let res = verify_with_ecdsa(&public_key, &"114514".to_string(), &signature).unwrap();
    assert!(res);

    let res = verify_with_ecdsa(
        &public_key,
        &"114514".to_string(),
        &"eHmzHbsBLeMq7uXkEpwNVruztSl0rQ1417CxxwdS3H/IOtn0N77MsgaZszNxDkOtP0kO0bz/t0+no+V2G/eiDQ=="
            .to_string(),
    )
    .unwrap();
    assert_eq!(res, false);

    let res = verify_with_ecdsa(
        &public_key,
        &"19810".to_string(),
        &"lrpkTJOdhdbzNMqklJIFxBLMmT6PIRggdEoW99XhKdbABOVasBGNH8LGaK7Ry6bvTQbhqMd/gn7Knul38weJAQ=="
            .to_string(),
    )
    .unwrap();
    assert_eq!(res, false);
}
