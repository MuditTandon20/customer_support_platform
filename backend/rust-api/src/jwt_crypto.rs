//! jsonwebtoken's supported provider interface, restricted to Aster's HS256.
//! ring performs HMAC signing and constant-time verification; token parsing,
//! claims, expiry and algorithm validation remain in jsonwebtoken.
use jsonwebtoken::{
    crypto::{CryptoProvider, JwkUtils, JwtSigner, JwtVerifier},
    errors::{ErrorKind, Result},
    Algorithm, DecodingKey, EncodingKey,
};
use ring::hmac;
use signature::{Signer, Verifier};
use std::sync::Once;

struct Hs256(hmac::Key);
impl Signer<Vec<u8>> for Hs256 {
    fn try_sign(&self, message: &[u8]) -> std::result::Result<Vec<u8>, signature::Error> {
        Ok(hmac::sign(&self.0, message).as_ref().to_vec())
    }
}
impl Verifier<Vec<u8>> for Hs256 {
    fn verify(&self, message: &[u8], tag: &Vec<u8>) -> std::result::Result<(), signature::Error> {
        hmac::verify(&self.0, message, tag).map_err(|_| signature::Error::new())
    }
}
impl JwtSigner for Hs256 {
    fn algorithm(&self) -> Algorithm {
        Algorithm::HS256
    }
}
impl JwtVerifier for Hs256 {
    fn algorithm(&self) -> Algorithm {
        Algorithm::HS256
    }
}
fn signer(algorithm: &Algorithm, key: &EncodingKey) -> Result<Box<dyn JwtSigner>> {
    if *algorithm != Algorithm::HS256 {
        return Err(ErrorKind::InvalidAlgorithm.into());
    }
    Ok(Box::new(Hs256(hmac::Key::new(
        hmac::HMAC_SHA256,
        key.try_get_hmac_secret()?,
    ))))
}
fn verifier(algorithm: &Algorithm, key: &DecodingKey) -> Result<Box<dyn JwtVerifier>> {
    if *algorithm != Algorithm::HS256 {
        return Err(ErrorKind::InvalidAlgorithm.into());
    }
    Ok(Box::new(Hs256(hmac::Key::new(
        hmac::HMAC_SHA256,
        key.try_get_hmac_secret()?,
    ))))
}
static PROVIDER: CryptoProvider = CryptoProvider {
    signer_factory: signer,
    verifier_factory: verifier,
    // Aster does not consume JWKs or asymmetric keys.
    jwk_utils: JwkUtils::new_unimplemented(),
};
pub fn install() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        PROVIDER
            .install_default()
            .expect("Aster JWT provider must be installed first")
    });
}
