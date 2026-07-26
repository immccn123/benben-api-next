use hmac::{Hmac, Mac, digest::KeyInit};
use p256::elliptic_curve::ff::PrimeField;
use p256::elliptic_curve::sec1::ToSec1Point;
use p256::{ProjectivePoint, PublicKey, Scalar, SecretKey};
use p256::ecdsa::{SigningKey, VerifyingKey};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

fn derive_il(root_pub_sec1: &[u8], app_id: &str) -> Scalar {
    let mut mac = HmacSha256::new_from_slice(root_pub_sec1).expect("HMAC accepts any key length");
    mac.update(b"sso-derive:");
    mac.update(app_id.as_bytes());
    let hash = mac.finalize().into_bytes();
    <Scalar as PrimeField>::from_repr(hash.into()).expect("valid P-256 scalar")
}

pub fn derive_verifying_key(root_pub: &PublicKey, app_id: &str) -> VerifyingKey {
    let sec1 = root_pub.as_affine().to_sec1_point(true);
    let il = derive_il(sec1.as_bytes(), app_id);
    let il_point = ProjectivePoint::GENERATOR * il;
    let child = ProjectivePoint::from(root_pub.as_affine()) + il_point;
    let child_pub = PublicKey::from_affine(child.to_affine()).expect("valid point");
    VerifyingKey::from(child_pub)
}

pub fn derive_signing_key(root_priv: &SecretKey, app_id: &str) -> SigningKey {
    let root_pub = root_priv.public_key();
    let sec1 = root_pub.as_affine().to_sec1_point(true);
    let il = derive_il(sec1.as_bytes(), app_id);
    let parent = root_priv.to_nonzero_scalar();
    let child_scalar = *parent.as_ref() + il;
    let child_nz = p256::NonZeroScalar::new(child_scalar).expect("non-zero scalar");
    SigningKey::from(SecretKey::from(child_nz))
}

pub fn root_pub_from_pem(pem: &str) -> Option<PublicKey> {
    pkcs8::DecodePublicKey::from_public_key_pem(pem).ok()
}

pub fn root_priv_from_pem(pem: &str) -> Option<SecretKey> {
    pkcs8::DecodePrivateKey::from_pkcs8_pem(pem).ok()
}

pub fn signing_key_to_pem(sk: &SigningKey) -> String {
    pkcs8::EncodePrivateKey::to_pkcs8_pem(sk, pkcs8::LineEnding::LF)
        .expect("encode signing key")
        .to_string()
}

pub fn verifying_key_to_pem(vk: &VerifyingKey) -> String {
    let pub_key: PublicKey = vk.clone().into();
    pkcs8::EncodePublicKey::to_public_key_pem(&pub_key, pkcs8::LineEnding::LF)
        .expect("encode verifying key")
}

pub fn public_key_to_pem(pk: &PublicKey) -> String {
    pkcs8::EncodePublicKey::to_public_key_pem(pk, pkcs8::LineEnding::LF)
        .expect("encode public key")
}

pub const JWT_DERIVE_ID: &str = "__jwt__";
