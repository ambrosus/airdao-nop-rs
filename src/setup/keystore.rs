use cipher::{InnerIvInit, KeyInit, StreamCipherCore};
use eth_keystore::{CipherparamsJson, CryptoJson, KdfType, KdfparamsType};
use k256::ecdsa::SigningKey;
use rand::{CryptoRng, Rng};
use scrypt::{scrypt, Params as ScryptParams};
use serde::{Deserialize, Serialize};
use sha3::{digest::Update, Digest, Keccak256};
use std::path::Path;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

const DEFAULT_CIPHER: &str = "aes-128-ctr";
const DEFAULT_KEY_SIZE: usize = 32usize;
const DEFAULT_IV_SIZE: usize = 16usize;
const DEFAULT_KDF_PARAMS_DKLEN: u8 = 32u8;
const DEFAULT_KDF_PARAMS_LOG_N: u8 = 17u8;
const DEFAULT_KDF_PARAMS_R: u32 = 8u32;
const DEFAULT_KDF_PARAMS_P: u32 = 1u32;

pub async fn encrypt_key<P, R, B, S>(
    dir: P,
    rng: &mut R,
    pk: B,
    password: S,
    name: Option<&str>,
) -> anyhow::Result<String>
where
    P: AsRef<Path>,
    R: Rng + CryptoRng,
    B: AsRef<[u8]>,
    S: AsRef<[u8]>,
{
    // Generate a random salt.
    let mut salt = vec![0u8; DEFAULT_KEY_SIZE];
    rng.fill_bytes(salt.as_mut_slice());

    // Derive the key.
    let mut key = vec![0u8; DEFAULT_KDF_PARAMS_DKLEN as usize];
    let scrypt_params = ScryptParams::new(
        DEFAULT_KDF_PARAMS_LOG_N,
        DEFAULT_KDF_PARAMS_R,
        DEFAULT_KDF_PARAMS_P,
        32,
    )?;
    scrypt(password.as_ref(), &salt, &scrypt_params, key.as_mut_slice())?;

    // Encrypt the private key using AES-128-CTR.
    let mut iv = vec![0u8; DEFAULT_IV_SIZE];
    rng.fill_bytes(iv.as_mut_slice());

    let encryptor = Aes128Ctr::new(&key[..16], &iv[..16]).expect("invalid length");

    let mut ciphertext = pk.as_ref().to_vec();
    encryptor.apply_keystream(&mut ciphertext);

    // Calculate the MAC.
    let mac = Keccak256::default()
        .chain(&key[16..32])
        .chain(&ciphertext)
        .finalize();

    // If a file name is not specified for the keystore, simply use the strigified uuid.
    let id = uuid::Uuid::new_v4();
    let name = if let Some(name) = name {
        name.to_string()
    } else {
        id.to_string()
    };

    // Construct and serialize the encrypted JSON keystore.
    let keystore = EthKeystore {
        id,
        version: 3,
        crypto: CryptoJson {
            cipher: String::from(DEFAULT_CIPHER),
            cipherparams: CipherparamsJson { iv },
            ciphertext: ciphertext.to_vec(),
            kdf: KdfType::Scrypt,
            kdfparams: KdfparamsType::Scrypt {
                dklen: DEFAULT_KDF_PARAMS_DKLEN,
                n: 2u32.pow(DEFAULT_KDF_PARAMS_LOG_N as u32),
                p: DEFAULT_KDF_PARAMS_P,
                r: DEFAULT_KDF_PARAMS_R,
                salt,
            },
            mac: mac.to_vec(),
        },
        address: hex::encode(crate::utils::secp256k1_signing_key_to_eth_address(
            &SigningKey::from_slice(pk.as_ref())?,
        )), // hex encoded eth address without `0x`` prefix
    };
    let contents = serde_json::to_string(&keystore)?;

    // Create a file in write-only mode, to store the encrypted JSON keystore.
    let mut file = tokio::fs::File::create(dir.as_ref().join(&name)).await?;
    file.write_all(contents.as_bytes()).await?;

    Ok(id.to_string())
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EthKeystore {
    pub address: String,
    pub crypto: CryptoJson,
    pub id: Uuid,
    pub version: u8,
}

struct Aes128Ctr {
    inner: ctr::CtrCore<aes::Aes128, ctr::flavors::Ctr128BE>,
}

impl Aes128Ctr {
    fn new(key: &[u8], iv: &[u8]) -> Result<Self, cipher::InvalidLength> {
        let cipher = aes::Aes128::new_from_slice(key).unwrap();
        let inner = ctr::CtrCore::inner_iv_slice_init(cipher, iv).unwrap();
        Ok(Self { inner })
    }

    fn apply_keystream(self, buf: &mut [u8]) {
        self.inner.apply_keystream_partial(buf.into());
    }
}
