use std::io::{Read, Seek, SeekFrom};

use aes::{Aes128, Aes192, Aes256};
use base64::prelude::{BASE64_STANDARD, Engine as _};
use cbc::cipher::{BlockModeDecrypt, KeyIvInit, block_padding::NoPadding};
use hmac::{Hmac, KeyInit, Mac};
use quick_xml::XmlVersion;
use quick_xml::events::{BytesStart, Event};
use quick_xml::name::{Namespace, ResolveResult};
use quick_xml::reader::NsReader;
use sha1::{Digest, Sha1};
use sha2::{Sha256, Sha384, Sha512};

use crate::error::{OpcError, Result};
use crate::package::PackageReadLimits;

const AGILE_MAJOR_VERSION: u16 = 4;
const AGILE_MINOR_VERSION: u16 = 4;
const AGILE_RESERVED: u32 = 0x40;
const ENCRYPTION_NS: &[u8] = b"http://schemas.microsoft.com/office/2006/encryption";
const PASSWORD_NS: &[u8] = b"http://schemas.microsoft.com/office/2006/keyEncryptor/password";
const PASSWORD_URI: &str = "http://schemas.microsoft.com/office/2006/keyEncryptor/password";
const MAX_ENCRYPTION_INFO_BYTES: u64 = 1_048_576;
const MAX_SALT_BYTES: usize = 65_536;
const MAX_SPIN_COUNT: u32 = 10_000_000;
const PACKAGE_SEGMENT_BYTES: usize = 4_096;

const VERIFIER_INPUT_BLOCK_KEY: [u8; 8] = [0xfe, 0xa7, 0xd2, 0x76, 0x3b, 0x4b, 0x9e, 0x79];
const VERIFIER_HASH_BLOCK_KEY: [u8; 8] = [0xd7, 0xaa, 0x0f, 0x6d, 0x30, 0x61, 0x34, 0x4e];
const PACKAGE_KEY_BLOCK_KEY: [u8; 8] = [0x14, 0x6e, 0x0b, 0xe7, 0xab, 0xac, 0xd0, 0xd6];
const HMAC_KEY_BLOCK_KEY: [u8; 8] = [0x5f, 0xb2, 0xad, 0x01, 0x0c, 0xb9, 0xe1, 0xf6];
const HMAC_VALUE_BLOCK_KEY: [u8; 8] = [0xa0, 0x67, 0x7f, 0x02, 0xb2, 0x2c, 0x84, 0x33];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HashAlgorithm {
    Sha1,
    Sha256,
    Sha384,
    Sha512,
}

enum HmacState {
    Sha1(Hmac<Sha1>),
    Sha256(Hmac<Sha256>),
    Sha384(Hmac<Sha384>),
    Sha512(Hmac<Sha512>),
}

impl HmacState {
    fn new(algorithm: HashAlgorithm, key: &[u8]) -> Result<Self> {
        macro_rules! initialize {
            ($digest:ty, $variant:ident) => {
                <Hmac<$digest> as KeyInit>::new_from_slice(key)
                    .map(Self::$variant)
                    .map_err(|_| OpcError::InvalidEncryptionInfo)
            };
        }
        match algorithm {
            HashAlgorithm::Sha1 => initialize!(Sha1, Sha1),
            HashAlgorithm::Sha256 => initialize!(Sha256, Sha256),
            HashAlgorithm::Sha384 => initialize!(Sha384, Sha384),
            HashAlgorithm::Sha512 => initialize!(Sha512, Sha512),
        }
    }

    fn update(&mut self, data: &[u8]) {
        match self {
            Self::Sha1(mac) => mac.update(data),
            Self::Sha256(mac) => mac.update(data),
            Self::Sha384(mac) => mac.update(data),
            Self::Sha512(mac) => mac.update(data),
        }
    }

    fn finalize(self) -> Vec<u8> {
        match self {
            Self::Sha1(mac) => mac.finalize().into_bytes().to_vec(),
            Self::Sha256(mac) => mac.finalize().into_bytes().to_vec(),
            Self::Sha384(mac) => mac.finalize().into_bytes().to_vec(),
            Self::Sha512(mac) => mac.finalize().into_bytes().to_vec(),
        }
    }
}

impl HashAlgorithm {
    fn parse(value: &str) -> Result<Self> {
        match value {
            "SHA1" | "SHA-1" => Ok(Self::Sha1),
            "SHA256" | "SHA-256" => Ok(Self::Sha256),
            "SHA384" | "SHA-384" => Ok(Self::Sha384),
            "SHA512" | "SHA-512" => Ok(Self::Sha512),
            _ => Err(OpcError::UnsupportedEncryptionAlgorithm(value.to_owned())),
        }
    }

    fn output_size(self) -> usize {
        match self {
            Self::Sha1 => 20,
            Self::Sha256 => 32,
            Self::Sha384 => 48,
            Self::Sha512 => 64,
        }
    }

    #[cfg(test)]
    fn name(self) -> &'static str {
        match self {
            Self::Sha1 => "SHA1",
            Self::Sha256 => "SHA256",
            Self::Sha384 => "SHA384",
            Self::Sha512 => "SHA512",
        }
    }

    fn digest(self, data: &[u8]) -> Vec<u8> {
        match self {
            Self::Sha1 => Sha1::digest(data).to_vec(),
            Self::Sha256 => Sha256::digest(data).to_vec(),
            Self::Sha384 => Sha384::digest(data).to_vec(),
            Self::Sha512 => Sha512::digest(data).to_vec(),
        }
    }

    #[cfg(test)]
    fn hmac(self, key: &[u8], data: &[u8]) -> Result<Vec<u8>> {
        let mut mac = HmacState::new(self, key)?;
        mac.update(data);
        Ok(mac.finalize())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CipherParameters {
    salt: Vec<u8>,
    key_bits: u16,
    hash: HashAlgorithm,
}

impl CipherParameters {
    fn from_element(element: &BytesStart<'_>) -> Result<Self> {
        let salt_size = parse_usize_attribute(element, b"saltSize")?;
        if salt_size == 0 || salt_size > MAX_SALT_BYTES {
            return Err(OpcError::InvalidEncryptionInfo);
        }
        if parse_usize_attribute(element, b"blockSize")? != 16 {
            return Err(OpcError::InvalidEncryptionInfo);
        }
        let key_bits = parse_u16_attribute(element, b"keyBits")?;
        if !matches!(key_bits, 128 | 192 | 256) {
            return Err(OpcError::InvalidEncryptionInfo);
        }
        if required_attribute(element, b"cipherAlgorithm")? != "AES"
            || required_attribute(element, b"cipherChaining")? != "ChainingModeCBC"
        {
            return Err(OpcError::UnsupportedEncryptionAlgorithm(
                "agile encryption requires AES-CBC".to_owned(),
            ));
        }
        let hash = HashAlgorithm::parse(&required_attribute(element, b"hashAlgorithm")?)?;
        if parse_usize_attribute(element, b"hashSize")? != hash.output_size() {
            return Err(OpcError::InvalidEncryptionInfo);
        }
        let salt = decode_attribute(element, b"saltValue")?;
        if salt.len() != salt_size {
            return Err(OpcError::InvalidEncryptionInfo);
        }
        Ok(Self {
            salt,
            key_bits,
            hash,
        })
    }

    fn key_bytes(&self) -> usize {
        usize::from(self.key_bits / 8)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PasswordKeyEncryptor {
    parameters: CipherParameters,
    spin_count: u32,
    encrypted_verifier_input: Vec<u8>,
    encrypted_verifier_hash: Vec<u8>,
    encrypted_package_key: Vec<u8>,
}

impl PasswordKeyEncryptor {
    fn from_element(element: &BytesStart<'_>) -> Result<Self> {
        let parameters = CipherParameters::from_element(element)?;
        let spin_count = parse_u32_attribute(element, b"spinCount")?;
        if spin_count > MAX_SPIN_COUNT {
            return Err(OpcError::InvalidEncryptionInfo);
        }
        let encrypted_verifier_input = decode_attribute(element, b"encryptedVerifierHashInput")?;
        let encrypted_verifier_hash = decode_attribute(element, b"encryptedVerifierHashValue")?;
        let encrypted_package_key = decode_attribute(element, b"encryptedKeyValue")?;
        validate_encrypted_field(
            &encrypted_verifier_input,
            round_up(parameters.salt.len(), 16)?,
        )?;
        validate_encrypted_field(
            &encrypted_verifier_hash,
            round_up(parameters.hash.output_size(), 16)?,
        )?;
        validate_encrypted_field(
            &encrypted_package_key,
            round_up(parameters.key_bytes(), 16)?,
        )?;
        Ok(Self {
            parameters,
            spin_count,
            encrypted_verifier_input,
            encrypted_verifier_hash,
            encrypted_package_key,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DataIntegrity {
    encrypted_hmac_key: Vec<u8>,
    encrypted_hmac_value: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EncryptionDescriptor {
    key_data: CipherParameters,
    integrity: DataIntegrity,
    password: PasswordKeyEncryptor,
}

impl EncryptionDescriptor {
    fn parse(stream: &[u8]) -> Result<Self> {
        if stream.len() < 8 {
            return Err(OpcError::InvalidEncryptionInfo);
        }
        let major = u16::from_le_bytes([stream[0], stream[1]]);
        let minor = u16::from_le_bytes([stream[2], stream[3]]);
        let reserved = u32::from_le_bytes([stream[4], stream[5], stream[6], stream[7]]);
        if major != AGILE_MAJOR_VERSION
            || minor != AGILE_MINOR_VERSION
            || reserved != AGILE_RESERVED
        {
            return Err(OpcError::UnsupportedEncryption(
                "only ECMA-376 agile encryption version 4.4 is supported",
            ));
        }
        let xml = &stream[8..];
        let xml = &xml[..xml
            .iter()
            .rposition(|byte| *byte != 0)
            .map_or(0, |index| index + 1)];
        parse_descriptor_xml(xml)
    }

    fn authenticate_password(&self, password: &str) -> Result<Vec<u8>> {
        if password.chars().count() > 255 {
            return Err(OpcError::InvalidPassword);
        }
        let base_hash = password_hash(password, &self.password)?;
        let verifier_key = derived_password_key(
            &base_hash,
            &VERIFIER_INPUT_BLOCK_KEY,
            &self.password.parameters,
        );
        let verifier_input = decrypt_aes_cbc(
            &self.password.encrypted_verifier_input,
            &verifier_key,
            &self.password.parameters.salt,
        )?;
        let verifier_hash_key = derived_password_key(
            &base_hash,
            &VERIFIER_HASH_BLOCK_KEY,
            &self.password.parameters,
        );
        let verifier_hash = decrypt_aes_cbc(
            &self.password.encrypted_verifier_hash,
            &verifier_hash_key,
            &self.password.parameters.salt,
        )?;
        let verifier_input = verifier_input
            .get(..self.password.parameters.salt.len())
            .ok_or(OpcError::InvalidEncryptionInfo)?;
        let expected = self.password.parameters.hash.digest(verifier_input);
        if !constant_time_eq(&verifier_hash[..expected.len()], &expected) {
            return Err(OpcError::InvalidPassword);
        }
        let package_key_key = derived_password_key(
            &base_hash,
            &PACKAGE_KEY_BLOCK_KEY,
            &self.password.parameters,
        );
        let package_key = decrypt_aes_cbc(
            &self.password.encrypted_package_key,
            &package_key_key,
            &self.password.parameters.salt,
        )?;
        Ok(package_key[..self.key_data.key_bytes()].to_vec())
    }

    fn authenticate_stream<R: Read>(&self, package_key: &[u8], mut encrypted: R) -> Result<()> {
        let hmac_key_iv = initialization_vector(
            &self.key_data.salt,
            Some(&HMAC_KEY_BLOCK_KEY),
            self.key_data.hash,
        );
        let hmac_key = decrypt_aes_cbc(
            &self.integrity.encrypted_hmac_key,
            package_key,
            &hmac_key_iv,
        )?;
        let hmac_value_iv = initialization_vector(
            &self.key_data.salt,
            Some(&HMAC_VALUE_BLOCK_KEY),
            self.key_data.hash,
        );
        let hmac_value = decrypt_aes_cbc(
            &self.integrity.encrypted_hmac_value,
            package_key,
            &hmac_value_iv,
        )?;
        let hmac_key_len =
            if self.integrity.encrypted_hmac_key.len() == round_up(self.key_data.salt.len(), 16)? {
                self.key_data.salt.len()
            } else {
                self.key_data.hash.output_size()
            };
        if hmac_key.len() < hmac_key_len || hmac_value.len() < self.key_data.hash.output_size() {
            return Err(OpcError::InvalidEncryptionInfo);
        }
        let mut mac = HmacState::new(self.key_data.hash, &hmac_key[..hmac_key_len])?;
        let mut buffer = [0_u8; 8_192];
        loop {
            let read = encrypted.read(&mut buffer)?;
            if read == 0 {
                break;
            }
            mac.update(&buffer[..read]);
        }
        let expected = mac.finalize();
        if !constant_time_eq(&hmac_value[..self.key_data.hash.output_size()], &expected) {
            return Err(OpcError::EncryptedPackageIntegrity);
        }
        Ok(())
    }
}

pub(crate) fn decrypt_package<R: Read + Seek>(
    reader: R,
    password: &str,
    limits: PackageReadLimits,
) -> Result<Vec<u8>> {
    let mut compound = cfb::CompoundFile::open(reader)?;
    let info_len = compound.entry("/EncryptionInfo")?.len();
    if info_len > MAX_ENCRYPTION_INFO_BYTES {
        return Err(OpcError::PackageLimitExceeded {
            kind: "encryption information size",
            limit: MAX_ENCRYPTION_INFO_BYTES,
        });
    }
    let mut info = Vec::with_capacity(usize::try_from(info_len).map_err(|_| {
        OpcError::PackageLimitExceeded {
            kind: "encryption information size",
            limit: MAX_ENCRYPTION_INFO_BYTES,
        }
    })?);
    compound
        .open_stream("/EncryptionInfo")?
        .read_to_end(&mut info)?;
    let descriptor = EncryptionDescriptor::parse(&info)?;
    let package_key = descriptor.authenticate_password(password)?;

    let encrypted_len = compound.entry("/EncryptedPackage")?.len();
    if encrypted_len < 8 {
        return Err(OpcError::InvalidEncryptedPackage);
    }
    let mut encrypted = compound.open_stream("/EncryptedPackage")?;
    descriptor.authenticate_stream(&package_key, &mut encrypted)?;
    encrypted.seek(SeekFrom::Start(0))?;
    decrypt_package_segments(
        &descriptor.key_data,
        &package_key,
        &mut encrypted,
        encrypted_len,
        limits,
    )
}

fn parse_descriptor_xml(xml: &[u8]) -> Result<EncryptionDescriptor> {
    let mut reader = NsReader::from_reader(xml);
    let mut buffer = Vec::new();
    let mut depth = 0_usize;
    let mut root_seen = false;
    let mut root_stage = 0_u8;
    let mut key_encryptor_uri = None;
    let mut key_data = None;
    let mut integrity = None;
    let mut password = None;

    loop {
        let (namespace, event) = reader.read_resolved_event_into(&mut buffer)?;
        let is_start = matches!(event, Event::Start(_));
        match &event {
            Event::Start(element) | Event::Empty(element) => {
                let local = element.local_name();
                let namespace = namespace_uri(&namespace)?;
                match depth {
                    0 if namespace == ENCRYPTION_NS && local.as_ref() == b"encryption" => {
                        if root_seen {
                            return Err(OpcError::InvalidEncryptionInfo);
                        }
                        root_seen = true;
                    }
                    1 if namespace == ENCRYPTION_NS && local.as_ref() == b"keyData" => {
                        if root_stage != 0 || is_start {
                            return Err(OpcError::InvalidEncryptionInfo);
                        }
                        key_data = Some(CipherParameters::from_element(element)?);
                        root_stage = 1;
                    }
                    1 if namespace == ENCRYPTION_NS && local.as_ref() == b"dataIntegrity" => {
                        if root_stage != 1 || is_start {
                            return Err(OpcError::InvalidEncryptionInfo);
                        }
                        integrity = Some(DataIntegrity {
                            encrypted_hmac_key: decode_attribute(element, b"encryptedHmacKey")?,
                            encrypted_hmac_value: decode_attribute(element, b"encryptedHmacValue")?,
                        });
                        root_stage = 2;
                    }
                    1 if namespace == ENCRYPTION_NS && local.as_ref() == b"keyEncryptors" => {
                        if root_stage != 2 || !is_start {
                            return Err(OpcError::InvalidEncryptionInfo);
                        }
                        root_stage = 3;
                    }
                    2 if namespace == ENCRYPTION_NS && local.as_ref() == b"keyEncryptor" => {
                        if key_encryptor_uri.is_some() || !is_start {
                            return Err(OpcError::InvalidEncryptionInfo);
                        }
                        let uri = required_attribute(element, b"uri")?;
                        if uri != PASSWORD_URI {
                            return Err(OpcError::UnsupportedEncryption(
                                "only password key encryptors are supported",
                            ));
                        }
                        key_encryptor_uri = Some(uri);
                    }
                    3 if namespace == PASSWORD_NS && local.as_ref() == b"encryptedKey" => {
                        if key_encryptor_uri.as_deref() != Some(PASSWORD_URI)
                            || password.is_some()
                            || is_start
                        {
                            return Err(OpcError::InvalidEncryptionInfo);
                        }
                        password = Some(PasswordKeyEncryptor::from_element(element)?);
                    }
                    _ => return Err(OpcError::InvalidEncryptionInfo),
                }
                if is_start {
                    depth = depth
                        .checked_add(1)
                        .ok_or(OpcError::InvalidEncryptionInfo)?;
                }
            }
            Event::End(_) => {
                depth = depth
                    .checked_sub(1)
                    .ok_or(OpcError::InvalidEncryptionInfo)?;
            }
            Event::Text(text) if text.iter().all(u8::is_ascii_whitespace) => {}
            Event::Decl(_) | Event::Comment(_) | Event::PI(_) => {}
            Event::Eof => break,
            _ => return Err(OpcError::InvalidEncryptionInfo),
        }
        buffer.clear();
    }

    if !root_seen || depth != 0 || root_stage != 3 {
        return Err(OpcError::InvalidEncryptionInfo);
    }
    let key_data = key_data.ok_or(OpcError::InvalidEncryptionInfo)?;
    let integrity = integrity.ok_or(OpcError::InvalidEncryptionInfo)?;
    let password = password.ok_or(OpcError::InvalidEncryptionInfo)?;
    let encrypted_hmac_key_len = integrity.encrypted_hmac_key.len();
    if encrypted_hmac_key_len != round_up(key_data.salt.len(), 16)?
        && encrypted_hmac_key_len != round_up(key_data.hash.output_size(), 16)?
    {
        return Err(OpcError::InvalidEncryptionInfo);
    }
    validate_encrypted_field(
        &integrity.encrypted_hmac_value,
        round_up(key_data.hash.output_size(), 16)?,
    )?;
    validate_encrypted_field(
        &password.encrypted_package_key,
        round_up(key_data.key_bytes(), 16)?,
    )?;
    Ok(EncryptionDescriptor {
        key_data,
        integrity,
        password,
    })
}

fn namespace_uri<'a>(namespace: &'a ResolveResult<'a>) -> Result<&'a [u8]> {
    match namespace {
        ResolveResult::Bound(Namespace(uri)) => Ok(uri),
        ResolveResult::Unbound | ResolveResult::Unknown(_) => Err(OpcError::InvalidEncryptionInfo),
    }
}

fn required_attribute(element: &BytesStart<'_>, expected: &[u8]) -> Result<String> {
    for attribute in element.attributes() {
        let attribute = attribute?;
        if attribute.key.as_ref() == expected {
            return Ok(attribute
                .decoded_and_normalized_value(XmlVersion::Implicit1_0, element.decoder())?
                .into_owned());
        }
    }
    Err(OpcError::InvalidEncryptionInfo)
}

fn parse_usize_attribute(element: &BytesStart<'_>, name: &[u8]) -> Result<usize> {
    required_attribute(element, name)?
        .parse()
        .map_err(|_| OpcError::InvalidEncryptionInfo)
}

fn parse_u16_attribute(element: &BytesStart<'_>, name: &[u8]) -> Result<u16> {
    required_attribute(element, name)?
        .parse()
        .map_err(|_| OpcError::InvalidEncryptionInfo)
}

fn parse_u32_attribute(element: &BytesStart<'_>, name: &[u8]) -> Result<u32> {
    required_attribute(element, name)?
        .parse()
        .map_err(|_| OpcError::InvalidEncryptionInfo)
}

fn decode_attribute(element: &BytesStart<'_>, name: &[u8]) -> Result<Vec<u8>> {
    let value = required_attribute(element, name)?;
    BASE64_STANDARD
        .decode(value)
        .map_err(|_| OpcError::InvalidEncryptionInfo)
}

fn validate_encrypted_field(value: &[u8], expected_len: usize) -> Result<()> {
    if value.len() != expected_len || !value.len().is_multiple_of(16) {
        return Err(OpcError::InvalidEncryptionInfo);
    }
    Ok(())
}

fn round_up(value: usize, block_size: usize) -> Result<usize> {
    value
        .checked_add(block_size - 1)
        .map(|sum| sum / block_size * block_size)
        .ok_or(OpcError::InvalidEncryptionInfo)
}

fn password_hash(password: &str, encryptor: &PasswordKeyEncryptor) -> Result<Vec<u8>> {
    let mut utf16 = Vec::with_capacity(password.len().saturating_mul(2));
    for unit in password.encode_utf16() {
        utf16.extend_from_slice(&unit.to_le_bytes());
    }
    let mut input = Vec::with_capacity(encryptor.parameters.salt.len() + utf16.len());
    input.extend_from_slice(&encryptor.parameters.salt);
    input.extend_from_slice(&utf16);
    let mut hash = encryptor.parameters.hash.digest(&input);
    let mut iteration_input = Vec::with_capacity(4 + hash.len());
    for iteration in 0..encryptor.spin_count {
        iteration_input.clear();
        iteration_input.extend_from_slice(&iteration.to_le_bytes());
        iteration_input.extend_from_slice(&hash);
        hash = encryptor.parameters.hash.digest(&iteration_input);
    }
    Ok(hash)
}

fn derived_password_key(
    password_hash: &[u8],
    block_key: &[u8],
    parameters: &CipherParameters,
) -> Vec<u8> {
    let mut input = Vec::with_capacity(password_hash.len() + block_key.len());
    input.extend_from_slice(password_hash);
    input.extend_from_slice(block_key);
    let mut key = parameters.hash.digest(&input);
    key.resize(parameters.key_bytes(), 0x36);
    key.truncate(parameters.key_bytes());
    key
}

fn initialization_vector(salt: &[u8], block_key: Option<&[u8]>, hash: HashAlgorithm) -> Vec<u8> {
    let mut iv = if let Some(block_key) = block_key {
        let mut input = Vec::with_capacity(salt.len() + block_key.len());
        input.extend_from_slice(salt);
        input.extend_from_slice(block_key);
        hash.digest(&input)
    } else {
        salt.to_vec()
    };
    iv.resize(16, 0x36);
    iv.truncate(16);
    iv
}

fn decrypt_aes_cbc(ciphertext: &[u8], key: &[u8], iv: &[u8]) -> Result<Vec<u8>> {
    if ciphertext.is_empty() || !ciphertext.len().is_multiple_of(16) || iv.len() != 16 {
        return Err(OpcError::InvalidEncryptionInfo);
    }
    let mut plaintext = ciphertext.to_vec();
    macro_rules! decrypt {
        ($cipher:ty) => {
            cbc::Decryptor::<$cipher>::new_from_slices(key, iv)
                .map_err(|_| OpcError::InvalidEncryptionInfo)?
                .decrypt_padded::<NoPadding>(&mut plaintext)
                .map_err(|_| OpcError::InvalidEncryptionInfo)?
        };
    }
    match key.len() {
        16 => {
            decrypt!(Aes128);
        }
        24 => {
            decrypt!(Aes192);
        }
        32 => {
            decrypt!(Aes256);
        }
        _ => return Err(OpcError::InvalidEncryptionInfo),
    }
    Ok(plaintext)
}

fn decrypt_package_segments<R: Read>(
    parameters: &CipherParameters,
    package_key: &[u8],
    mut encrypted: R,
    encrypted_len: u64,
    limits: PackageReadLimits,
) -> Result<Vec<u8>> {
    let mut size_bytes = [0_u8; 8];
    read_exact_encrypted(&mut encrypted, &mut size_bytes)?;
    let plaintext_len_u64 = u64::from_le_bytes(size_bytes);
    if plaintext_len_u64 > limits.max_total_uncompressed_bytes {
        return Err(OpcError::PackageLimitExceeded {
            kind: "encrypted package plaintext size",
            limit: limits.max_total_uncompressed_bytes,
        });
    }
    let plaintext_len =
        usize::try_from(plaintext_len_u64).map_err(|_| OpcError::PackageLimitExceeded {
            kind: "encrypted package plaintext size",
            limit: usize::MAX as u64,
        })?;
    let expected_ciphertext = encrypted_package_ciphertext_len(plaintext_len)?;
    if encrypted_len != u64::try_from(expected_ciphertext.saturating_add(8)).unwrap_or(u64::MAX) {
        return Err(OpcError::InvalidEncryptedPackage);
    }

    let mut plaintext = Vec::with_capacity(plaintext_len);
    let mut plaintext_offset = 0_usize;
    let mut segment = 0_u32;
    while plaintext_offset < plaintext_len {
        let clear_len = (plaintext_len - plaintext_offset).min(PACKAGE_SEGMENT_BYTES);
        let encrypted_len = round_up(clear_len, 16)?;
        let mut ciphertext = vec![0_u8; encrypted_len];
        read_exact_encrypted(&mut encrypted, &mut ciphertext)?;
        let iv = initialization_vector(
            &parameters.salt,
            Some(&segment.to_le_bytes()),
            parameters.hash,
        );
        let clear = decrypt_aes_cbc(&ciphertext, package_key, &iv)?;
        plaintext.extend_from_slice(
            clear
                .get(..clear_len)
                .ok_or(OpcError::InvalidEncryptedPackage)?,
        );
        plaintext_offset += clear_len;
        segment = segment
            .checked_add(1)
            .ok_or(OpcError::InvalidEncryptedPackage)?;
    }
    Ok(plaintext)
}

fn read_exact_encrypted(reader: &mut impl Read, buffer: &mut [u8]) -> Result<()> {
    reader
        .read_exact(buffer)
        .map_err(|_| OpcError::InvalidEncryptedPackage)
}

fn encrypted_package_ciphertext_len(plaintext_len: usize) -> Result<usize> {
    let full_segments = plaintext_len / PACKAGE_SEGMENT_BYTES;
    let remainder = plaintext_len % PACKAGE_SEGMENT_BYTES;
    let full_len = full_segments
        .checked_mul(PACKAGE_SEGMENT_BYTES)
        .ok_or(OpcError::InvalidEncryptedPackage)?;
    if remainder == 0 {
        Ok(full_len)
    } else {
        full_len
            .checked_add(round_up(remainder, 16)?)
            .ok_or(OpcError::InvalidEncryptedPackage)
    }
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    left.iter()
        .zip(right)
        .fold(0_u8, |difference, (left, right)| {
            difference | (left ^ right)
        })
        == 0
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Read as _, Seek as _, SeekFrom, Write};

    use base64::prelude::{BASE64_STANDARD, Engine as _};
    use cbc::cipher::{BlockModeEncrypt, KeyIvInit, block_padding::NoPadding};

    use super::*;

    const PASSWORD: &str = "rdocx-f169";
    const MINIMAL_CONTENT_TYPES: &[u8] =
        br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="bin" ContentType="application/octet-stream"/>
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/custom/unmodelled.xml" ContentType="application/vnd.rdocx.test+xml"/>
</Types>"#;
    const PACKAGE_RELATIONSHIPS: &[u8] =
        br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="https://example.com/relationships/preserved" Target="custom/unmodelled.xml"/>
</Relationships>"#;
    const PART_RELATIONSHIPS: &[u8] = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId7" Type="https://example.com/relationships/opaque" Target="unmodelled.bin"/>
</Relationships>"#;
    const UNMODELLED_XML: &[u8] =
        br#"<x:root xmlns:x="https://example.com/opaque"><x:child flag="keep">raw</x:child></x:root>"#;

    #[test]
    fn agile_parameters_reject_unknown_or_inconsistent_algorithms() {
        let descriptor = test_descriptor_xml("e", "p", "SHA512", 64, 256, 16);
        assert!(parse_descriptor_xml(descriptor.as_bytes()).is_ok());

        let aliased = test_descriptor_xml("alias", "secret", "SHA-256", 32, 128, 16);
        assert!(parse_descriptor_xml(aliased.as_bytes()).is_ok());

        let bad_hash_size = test_descriptor_xml("e", "p", "SHA512", 32, 256, 16);
        assert!(matches!(
            parse_descriptor_xml(bad_hash_size.as_bytes()),
            Err(OpcError::InvalidEncryptionInfo)
        ));
        let bad_key_size = test_descriptor_xml("e", "p", "SHA512", 64, 64, 16);
        assert!(matches!(
            parse_descriptor_xml(bad_key_size.as_bytes()),
            Err(OpcError::InvalidEncryptionInfo)
        ));
        let bad_block_size = test_descriptor_xml("e", "p", "SHA512", 64, 256, 8);
        assert!(matches!(
            parse_descriptor_xml(bad_block_size.as_bytes()),
            Err(OpcError::InvalidEncryptionInfo)
        ));
        let unknown_hash = test_descriptor_xml("e", "p", "SHA3", 64, 256, 16);
        assert!(matches!(
            parse_descriptor_xml(unknown_hash.as_bytes()),
            Err(OpcError::UnsupportedEncryptionAlgorithm(_))
        ));
        let wrong_order = swap_empty_elements(&descriptor, "<e:keyData", "<e:dataIntegrity");
        assert!(parse_descriptor_xml(wrong_order.as_bytes()).is_err());
        let mismatched_salt = descriptor.replacen("saltSize=\"16\"", "saltSize=\"17\"", 1);
        assert!(matches!(
            parse_descriptor_xml(mismatched_salt.as_bytes()),
            Err(OpcError::InvalidEncryptionInfo)
        ));
        let oversized_salt = descriptor.replacen("saltSize=\"16\"", "saltSize=\"65537\"", 1);
        assert!(matches!(
            parse_descriptor_xml(oversized_salt.as_bytes()),
            Err(OpcError::InvalidEncryptionInfo)
        ));
        let excessive_spin = descriptor.replacen("spinCount=\"1000\"", "spinCount=\"10000001\"", 1);
        assert!(matches!(
            parse_descriptor_xml(excessive_spin.as_bytes()),
            Err(OpcError::InvalidEncryptionInfo)
        ));

        for hash in [
            HashAlgorithm::Sha1,
            HashAlgorithm::Sha256,
            HashAlgorithm::Sha384,
            HashAlgorithm::Sha512,
        ] {
            for key_bits in [128, 192, 256] {
                let package = encrypted_test_package_with(PASSWORD, key_bits, hash);
                assert!(
                    crate::OpcPackage::from_encrypted_reader(Cursor::new(package), PASSWORD)
                        .is_ok(),
                    "{} with AES-{key_bits} must decrypt",
                    hash.name()
                );
            }
        }
    }

    #[test]
    fn wrong_password_never_releases_a_package_key() {
        let package = encrypted_test_package(PASSWORD);
        let error = decrypt_package(
            Cursor::new(package),
            "not-the-password",
            PackageReadLimits::UNBOUNDED,
        )
        .unwrap_err();
        assert!(matches!(error, OpcError::InvalidPassword));
    }

    #[test]
    fn word_agile_document_opens_only_with_its_password() {
        // Microsoft Word for Mac 16.104 is the pinned manual openability oracle.
        let package = BASE64_STANDARD
            .decode(concat!(
            "0M8R4KGxGuEAAAAAAAAAAAAAAAAAAAAAPgADAP7/CQAGAAAAAAAAAAAAAAADAAAAAQAAAAAAAAAAEAAAAgAAAAEAAAD+////AAAAAAAAAAAHAAAACAAAAP//",
            "////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////",
            "////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////",
            "////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////",
            "////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////",
            "///////////////////////////////////////////////////////////////////////////////////9////BAAAAP7///8GAAAABQAAAP7///8xAAAA",
            "/f////3///8KAAAACwAAAAwAAAANAAAADgAAAA8AAAAQAAAAEQAAABIAAAATAAAAFAAAABUAAAAWAAAAFwAAABgAAAAZAAAAGgAAABsAAAAcAAAAHQAAAB4A",
            "AAAfAAAAIAAAACEAAAAiAAAAIwAAACQAAAAlAAAAJgAAACcAAAAoAAAAKQAAACoAAAArAAAALAAAAC0AAAAuAAAALwAAADAAAAD+////MgAAAP7/////////",
            "////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////",
            "////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////",
            "////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////",
            "/////////////////////////////////////////////1IAbwBvAHQAIABFAG4AdAByAHkAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            "AAAAAAAAAAAWAAUA//////////8KAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACA1HeXTMd0BAwAAAIAHAAAAAAAARQBuAGMAcgB5AHAAdABlAGQA",
            "UABhAGMAawBhAGcAZQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACIAAgD///////////////8AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            "AAAAAAAAAAAJAAAAaE8AAAAAAAAGAEQAYQB0AGEAUwBwAGEAYwBlAHMAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAGAABAP//",
            "////////BAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAsLIZ5dMx3QHgJxrl0zHdAQAAAAAAAAAAAAAAAFYAZQByAHMAaQBvAG4AAAAAAAAAAAAAAAAAAAAAAAAA",
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAQAAIB////////////////AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAEwA",
            "AAAAAAAAAQAAAP7///8DAAAA/v////7///8GAAAABwAAAAgAAAD+////CgAAAAsAAAAMAAAADQAAAA4AAAAPAAAAEAAAABEAAAASAAAAEwAAABQAAAAVAAAA",
            "FgAAABcAAAAYAAAAGQAAABoAAAAbAAAAHAAAAB0AAAD+////////////////////////////////////////////////////////////////////////////",
            "////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////",
            "////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////",
            "////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////",
            "//////////////////////////////////////////////////////////////////////////////////////////88AAAATQBpAGMAcgBvAHMAbwBmAHQA",
            "LgBDAG8AbgB0AGEAaQBuAGUAcgAuAEQAYQB0AGEAUwBwAGEAYwBlAHMAAQAAAAEAAAABAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            "AAAAAAAAAAAAAAAAAAAAAAgAAAABAAAAaAAAAAEAAAAAAAAAIAAAAEUAbgBjAHIAeQBwAHQAZQBkAFAAYQBjAGsAYQBnAGUAMgAAAFMAdAByAG8AbgBnAEUA",
            "bgBjAHIAeQBwAHQAaQBvAG4ARABhAHQAYQBTAHAAYQBjAGUAAAAAAAAAAAAAAAAAAAAAAAAACAAAAAEAAAAyAAAAUwB0AHIAbwBuAGcARQBuAGMAcgB5AHAA",
            "dABpAG8AbgBUAHIAYQBuAHMAZgBvAHIAbQAAAFgAAAABAAAATAAAAHsARgBGADkAQQAzAEYAMAAzAC0ANQA2AEUARgAtADQANgAxADMALQBCAEQARAA1AC0A",
            "NQBBADQAMQBDADEARAAwADcAMgA0ADYAfQBOAAAATQBpAGMAcgBvAHMAbwBmAHQALgBDAG8AbgB0AGEAaQBuAGUAcgAuAEUAbgBjAHIAeQBwAHQAaQBvAG4A",
            "VAByAGEAbgBzAGYAbwByAG0AAAABAAAAAQAAAAEAAAAAAAAAAAAAAEQAYQB0AGEAUwBwAGEAYwBlAE0AYQBwAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            "AAAAAAAAAAAAAAAAAAAaAAIBAwAAAAUAAAD/////AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAgAAAHAAAAAAAAAARABhAHQAYQBTAHAA",
            "YQBjAGUASQBuAGYAbwAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABwAAQH/////BwAAAAYAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAMDZ",
            "GeXTMd0BwNkZ5dMx3QEAAAAAAAAAAAAAAABTAHQAcgBvAG4AZwBFAG4AYwByAHkAcAB0AGkAbwBuAEQAYQB0AGEAUwBwAGEAYwBlAAAAAAAAAAAAAAAAAAAA",
            "NAACAf///////////////wAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAQAAABAAAAAAAAAAFQAcgBhAG4AcwBmAG8AcgBtAEkAbgBmAG8A",
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAcAAEA//////////8IAAAAAAAAAAAAAAAAAAAAAAAAAAAAAADA2Rnl0zHdAeAnGuXTMd0B",
            "AAAAAAAAAAAAAAAAUwB0AHIAbwBuAGcARQBuAGMAcgB5AHAAdABpAG8AbgBUAHIAYQBuAHMAZgBvAHIAbQAAAAAAAAAAAAAAAAAAADQAAQH//////////wkA",
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAMDZGeXTMd0B4Cca5dMx3QEAAAAAAAAAAAAAAAAGAFAAcgBpAG0AYQByAHkAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAEgACAf///////////////wAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAUAAADIAAAAAAAAAEUA",
            "bgBjAHIAeQBwAHQAaQBvAG4ASQBuAGYAbwAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAeAAIBAgAAAAEAAAD/////AAAAAAAAAAAAAAAA",
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACQAAAAkFAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            "AAAAAAAAAAAAAAAAAAD///////////////8AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAAAAAAAAAAAAAA",
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAEAEAAAAA8P3htbCB2ZXJzaW9uPSIxLjAiIGVuY29kaW5nPSJVVEYt",
            "OCIgc3RhbmRhbG9uZT0ieWVzIj8+DQo8ZW5jcnlwdGlvbiB4bWxucz0iaHR0cDovL3NjaGVtYXMubWljcm9zb2Z0LmNvbS9vZmZpY2UvMjAwNi9lbmNyeXB0",
            "aW9uIiB4bWxuczpwPSJodHRwOi8vc2NoZW1hcy5taWNyb3NvZnQuY29tL29mZmljZS8yMDA2L2tleUVuY3J5cHRvci9wYXNzd29yZCIgeG1sbnM6Yz0iaHR0",
            "cDovL3NjaGVtYXMubWljcm9zb2Z0LmNvbS9vZmZpY2UvMjAwNi9rZXlFbmNyeXB0b3IvY2VydGlmaWNhdGUiPjxrZXlEYXRhIHNhbHRTaXplPSIxNiIgYmxv",
            "Y2tTaXplPSIxNiIga2V5Qml0cz0iMjU2IiBoYXNoU2l6ZT0iNjQiIGNpcGhlckFsZ29yaXRobT0iQUVTIiBjaXBoZXJDaGFpbmluZz0iQ2hhaW5pbmdNb2Rl",
            "Q0JDIiBoYXNoQWxnb3JpdGhtPSJTSEE1MTIiIHNhbHRWYWx1ZT0iRkpXZEN1V///////////////////////////////////////////////////////////",
            "////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////",
            "////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////",
            "////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////",
            "////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////",
            "////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////",
            "////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////",
            "////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////",
            "////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////",
            "////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////",
            "////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////",
            "//////////////////////////////////////////////////////////////////////////////////////////////////////////9STwAAAAAAACGy",
            "dEkV/6nRSf0Myn0h1vN7Sn7bwLy0Hqog2KzdFNZQpOlq4QtMSI3cOrAZi6Acf/ka7+OnCZiq8YyqaD+a7YBNTT5VCl26L3lrsetQII+zMtJFCTWWf/J/NqMX",
            "/i7zNHweS3GjKKhB3kGx9IqaWq9w7/T1x90gkW9btji6ChG5khvpt22G041Q4k/dhVxBlo88wF3u6TamSK9JwpveLoLMs5QaysRxlNG+Eg/JvPFsrxOXPbPP",
            "pmB6Lul+xqkNmaPo+Slu8+LZLd/htTK5tGI1sDkjafceEe+JAnoGG0Vgl02EewXllc+orqzB6sswmBYfwukVOnFXkn76a4vSd3BUGSz/zoy28izb8ya+8uip",
            "fB5cXbzZ63c7b4bwiPHxncOqFNq1upqgEyRWXw64spIm2zoII0AJTk9xAcpNHZAKeBMzP3c3IkN9NYInjuWT0j1pw/VIfyyNQCr2YFIV2ck5wY6zQE9TUFa3",
            "Jt1WSfgaTgMNKAenyzJHkg9CZeuQOGNiovSJ92Pg5BcdwU8ypZun7ER5Ga4DWoc6ii6XS9BRqAL8shka4hkhQAUiGndTWz1Jm7InLNZH/NMmgZx3rfTgl/bi",
            "vByU4gw9N/FeEX0EgNUA6ZeqD0LpljfhuRlFh6Ux8Eh2itbk1bhLl9XpHufmyPxaqBjFUapG6TSgdKcKRlmhIN5rVsfyjZSJnXlg/MwlWI5PvWBy9MWuJ49L",
            "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",
            "m85dVz5+vogyV6TRo18ShMh10kf5dU8rWjiLQJjUSe14wVjDtnm5YS3TtfISxpdzUt2GBUswQywKRCs3yeqrx1wJBOPCCw9T68V5Sn5pGC1zyKTcXEGe+TC/",
            "ebval6B1V8KwOq2KMDgsJmNpRTRqRyRhjgva3nQiCpYBRcqq9UTYEuCenDuwAGXjbhg3+mB3bdgOqqJzRB/D2YSg/8Ps3JJgxNGq9X5R/RHCiVoRoyMYgPlo",
            "ZoPBlT7tUKJNejJxOkHYwe/SnrONwaXcyL7yyWI4iUuTJAQv56Ta1f73xFPFoZFvHdhHcEStB2hk1muBbG9HrTzmPUVVoMKeWQ1IQ9Q7vRFRHE5SVaDQa3Uk",
            "jnxiYJBX24zZmuE0f4ap0tHshrgLfkwRI4UwRgyHwp2hhKw3RJwCH2HAElJw/8zCWJl79g7N4CrqP4elbKba1JfZ+AntZJktzd7WUp9d1OG1f5RV381KQgoa",
            "BG4oQji1iXNIxQ8Cc8/4Fssshlw/wtptDvm6denkRhraOrP1NzndPxO0tshLtwnW1AGSUyCxHIk+/hKckhVX6nXrV9nW1bmEPOX32psX8Yi7mBJkYyrQrhSw",
            "Ynp+eInvnANZjz60b/76wFS+z1yDsM9fdalI6SlM30ILn375ayuMh0ZYQzZkeitHL2EzalQ2QT09Ii8+PGRhdGFJbnRlZ3JpdHkgZW5jcnlwdGVkSG1hY0tl",
            "eT0ibk10NmY5SDRMb29xQjFRd05jOWlPTEM5aGQ5YWJCQ054K1l0L01HVDBRUkJvQThaWDk5QkpoenJTeUNobThwSkRVU0FvTHpGaVhiNUFtY0dDUXIzSWc9",
            "PSIgZW5jcnlwdGVkSG1hY1ZhbHVlPSJna3VqTG5EZ0dTQjBqYTdDM2tGK1I5OTE2d0cwMm11RWNEejhDUnVVRmYvUEZiS1lieWZweGJlbEFqV1ZVZ3dON2h4",
            "MVdXQzRBOFVZTHpYMDlpaVRpUT09Ii8+PGtleUVuY3J5cHRvcnM+PGtleUVuY3J5cHRvciB1cmk9Imh0dHA6Ly9zY2hlbWFzLm1pY3Jvc29mdC5jb20vb2Zm",
            "aWNlLzIwMDYva2V5RW5jcnlwdG9yL3Bhc3N3b3JkIj48cDplbmNyeXB0ZWRLZXkgc3BpbkNvdW50PSIxMDAwMDAiIHNhbHRTaXplPSIxNiIgYmxvY2tTaXpl",
            "PSIxNiIga2V5Qml0cz0iMjU2IiBoYXNoU2l6ZT0iNjQiIGNpcGhlckFsZ29yaXRobT0iQUVTIiBjaXBoZXJDaGFpbmluZz0iQ2hhaW5pbmdNb2RlQ0JDIiBo",
            "YXNoQWxnb3JpdGhtPSJTSEE1MTIiIHNhbHRWYWx1ZT0iYzdjTXRTTThvTG44VWkzcWcrMkRrUT09IiBlbmNyeXB0ZWRWZXJpZmllckhhc2hJbnB1dD0iT1hG",
            "Wm1tb3FpeEY1c3dab3hQd1ZiQT09IiBlbmNyeXB0ZWRWZXJpZmllckhhc2hWYWx1ZT0iMldnTUhSNm01OThUOEs4SjlUYmZzVXZNNU5TczVZU2trdkQ5M1o0",
            "dHVJZE9WVlljZUpadUxqQUtvS0hXNDI5TGVHUVR3QUlhRXZGSjMxZ0VVMzcyamc9PSIgZW5jcnlwdGVkS2V5VmFsdWU9Ik1HbTFyMVZLL0o3OVE2dG9YdXRC",
            "R243emZWdFNWcU1Ud2p2cEJTS2pPMGM9Ii8+PC9rZXlFbmNyeXB0b3I+PC9rZXlFbmNyeXB0b3JzPjwvZW5jcnlwdGlvbj4AAAAAAAAAAAAAAAAAAAAAAAAA",
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=",
            ))
            .unwrap();
        let opened =
            crate::OpcPackage::from_encrypted_reader(Cursor::new(&package), PASSWORD).unwrap();
        assert!(opened.main_document_part().is_some());
        assert!(matches!(
            crate::OpcPackage::from_encrypted_reader(Cursor::new(&package), "wrong"),
            Err(OpcError::InvalidPassword)
        ));
    }

    #[test]
    fn tampered_agile_package_fails_before_zip_parsing() {
        let ciphertext_tampered =
            tamper_encrypted_package_ciphertext(encrypted_test_package(PASSWORD));
        assert_integrity_failure(ciphertext_tampered);

        let hmac_tampered = tamper_encrypted_hmac_value(encrypted_test_package(PASSWORD));
        assert_integrity_failure(hmac_tampered);
    }

    #[test]
    fn decrypted_package_preserves_every_unrelated_part() {
        let zip = test_zip();
        let package = encrypted_test_package_from_zip(PASSWORD, &zip);
        let rejected = crate::OpcPackage::from_encrypted_reader_with_limits(
            Cursor::new(&package),
            PASSWORD,
            PackageReadLimits {
                max_entries: 16,
                max_part_uncompressed_bytes: 4_096,
                max_total_uncompressed_bytes: zip.len() as u64 - 1,
            },
        )
        .unwrap_err();
        assert!(matches!(rejected, OpcError::PackageLimitExceeded { .. }));

        let parsed = crate::OpcPackage::from_encrypted_reader_with_limits(
            Cursor::new(package),
            PASSWORD,
            PackageReadLimits {
                max_entries: 16,
                max_part_uncompressed_bytes: 4_096,
                max_total_uncompressed_bytes: zip.len() as u64,
            },
        )
        .unwrap();
        assert_eq!(
            parsed.get_part("/custom/unmodelled.bin"),
            Some(&b"opaque payload"[..])
        );
        assert_eq!(
            parsed.get_part("/custom/unmodelled.xml"),
            Some(UNMODELLED_XML)
        );
        assert_eq!(
            parsed
                .content_types
                .content_type_for("/custom/unmodelled.xml"),
            Some("application/vnd.rdocx.test+xml")
        );
        assert_eq!(parsed.package_rels.items[0].target, "custom/unmodelled.xml");
        assert_eq!(
            parsed
                .get_part_rels("/custom/unmodelled.xml")
                .unwrap()
                .items[0]
                .target,
            "unmodelled.bin"
        );
        let mut saved = Cursor::new(Vec::new());
        parsed.write_to(&mut saved).unwrap();
        let reparsed = crate::OpcPackage::from_reader(Cursor::new(saved.into_inner())).unwrap();
        assert_eq!(
            reparsed.get_part("/custom/unmodelled.bin"),
            Some(&b"opaque payload"[..])
        );
        assert_eq!(
            reparsed.get_part("/custom/unmodelled.xml"),
            Some(UNMODELLED_XML)
        );
        assert_eq!(
            reparsed
                .content_types
                .content_type_for("/custom/unmodelled.xml"),
            Some("application/vnd.rdocx.test+xml")
        );
        assert_eq!(
            reparsed.package_rels.items[0].target,
            "custom/unmodelled.xml"
        );
        assert_eq!(
            reparsed
                .get_part_rels("/custom/unmodelled.xml")
                .unwrap()
                .items[0]
                .target,
            "unmodelled.bin"
        );
    }

    fn test_zip() -> Vec<u8> {
        let mut output = Cursor::new(Vec::new());
        {
            let mut zip = zip::ZipWriter::new(&mut output);
            let options = zip::write::SimpleFileOptions::default();
            zip.start_file("[Content_Types].xml", options).unwrap();
            zip.write_all(MINIMAL_CONTENT_TYPES).unwrap();
            zip.start_file("_rels/.rels", options).unwrap();
            zip.write_all(PACKAGE_RELATIONSHIPS).unwrap();
            zip.start_file("custom/unmodelled.xml", options).unwrap();
            zip.write_all(UNMODELLED_XML).unwrap();
            zip.start_file("custom/_rels/unmodelled.xml.rels", options)
                .unwrap();
            zip.write_all(PART_RELATIONSHIPS).unwrap();
            zip.start_file("custom/unmodelled.bin", options).unwrap();
            zip.write_all(b"opaque payload").unwrap();
            zip.finish().unwrap();
        }
        output.into_inner()
    }

    fn tamper_encrypted_package_ciphertext(package: Vec<u8>) -> Vec<u8> {
        let mut compound = cfb::CompoundFile::open(Cursor::new(package)).unwrap();
        let mut stream = compound.open_stream("/EncryptedPackage").unwrap();
        stream.seek(SeekFrom::End(-1)).unwrap();
        let mut byte = [0_u8; 1];
        stream.read_exact(&mut byte).unwrap();
        stream.seek(SeekFrom::End(-1)).unwrap();
        byte[0] ^= 0x80;
        stream.write_all(&byte).unwrap();
        drop(stream);
        compound.flush().unwrap();
        compound.into_inner().into_inner()
    }

    fn tamper_encrypted_hmac_value(package: Vec<u8>) -> Vec<u8> {
        let mut compound = cfb::CompoundFile::open(Cursor::new(package)).unwrap();
        let mut stream = compound.open_stream("/EncryptionInfo").unwrap();
        let mut info = Vec::new();
        stream.read_to_end(&mut info).unwrap();
        let marker = b"encryptedHmacValue=\"";
        let marker_start = info
            .windows(marker.len())
            .position(|window| window == marker)
            .unwrap();
        let value_start = marker_start + marker.len();
        info[value_start] = if info[value_start] == b'A' {
            b'B'
        } else {
            b'A'
        };
        stream.seek(SeekFrom::Start(0)).unwrap();
        stream.write_all(&info).unwrap();
        drop(stream);
        compound.flush().unwrap();
        compound.into_inner().into_inner()
    }

    fn assert_integrity_failure(package: Vec<u8>) {
        let error = decrypt_package(Cursor::new(package), PASSWORD, PackageReadLimits::UNBOUNDED)
            .unwrap_err();
        assert!(matches!(error, OpcError::EncryptedPackageIntegrity));
    }

    fn swap_empty_elements(xml: &str, first: &str, second: &str) -> String {
        let first_start = xml.find(first).unwrap();
        let first_end = first_start + xml[first_start..].find("/>").unwrap() + 2;
        let second_start = xml.find(second).unwrap();
        let second_end = second_start + xml[second_start..].find("/>").unwrap() + 2;
        format!(
            "{}{}{}{}{}",
            &xml[..first_start],
            &xml[second_start..second_end],
            &xml[first_end..second_start],
            &xml[first_start..first_end],
            &xml[second_end..]
        )
    }

    fn encrypted_test_package(password: &str) -> Vec<u8> {
        encrypted_test_package_with(password, 256, HashAlgorithm::Sha512)
    }

    fn encrypted_test_package_with(password: &str, key_bits: u16, hash: HashAlgorithm) -> Vec<u8> {
        encrypted_test_package_from_zip_with(password, &test_zip(), key_bits, hash)
    }

    fn encrypted_test_package_from_zip(password: &str, zip: &[u8]) -> Vec<u8> {
        encrypted_test_package_from_zip_with(password, zip, 256, HashAlgorithm::Sha512)
    }

    fn encrypted_test_package_from_zip_with(
        password: &str,
        zip: &[u8],
        key_bits: u16,
        hash: HashAlgorithm,
    ) -> Vec<u8> {
        let key_data = CipherParameters {
            salt: (0_u8..16).collect(),
            key_bits,
            hash,
        };
        let password_parameters = CipherParameters {
            salt: (16_u8..32).collect(),
            key_bits,
            hash,
        };
        let package_key: Vec<u8> = (32_u8..).take(key_data.key_bytes()).collect();
        let verifier: Vec<u8> = (64_u8..80).collect();
        let spin_count = 1_000;
        let encryptor = PasswordKeyEncryptor {
            parameters: password_parameters.clone(),
            spin_count,
            encrypted_verifier_input: Vec::new(),
            encrypted_verifier_hash: Vec::new(),
            encrypted_package_key: Vec::new(),
        };
        let base_hash = password_hash(password, &encryptor).unwrap();
        let verifier_key =
            derived_password_key(&base_hash, &VERIFIER_INPUT_BLOCK_KEY, &password_parameters);
        let encrypted_verifier_input =
            encrypt_aes_cbc(&verifier, &verifier_key, &password_parameters.salt);
        let verifier_hash_key =
            derived_password_key(&base_hash, &VERIFIER_HASH_BLOCK_KEY, &password_parameters);
        let encrypted_verifier_hash = encrypt_aes_cbc(
            &zero_pad(password_parameters.hash.digest(&verifier)),
            &verifier_hash_key,
            &password_parameters.salt,
        );
        let package_key_key =
            derived_password_key(&base_hash, &PACKAGE_KEY_BLOCK_KEY, &password_parameters);
        let encrypted_package_key = encrypt_aes_cbc(
            &zero_pad(package_key.clone()),
            &package_key_key,
            &password_parameters.salt,
        );

        let mut encrypted_package = (zip.len() as u64).to_le_bytes().to_vec();
        for (segment, chunk) in zip.chunks(PACKAGE_SEGMENT_BYTES).enumerate() {
            let iv = initialization_vector(
                &key_data.salt,
                Some(&(segment as u32).to_le_bytes()),
                key_data.hash,
            );
            encrypted_package.extend_from_slice(&encrypt_aes_cbc(
                &zero_pad(chunk.to_vec()),
                &package_key,
                &iv,
            ));
        }

        let hmac_key: Vec<u8> = (80_u8..96).collect();
        let hmac_value = key_data.hash.hmac(&hmac_key, &encrypted_package).unwrap();
        let encrypted_hmac_key = encrypt_aes_cbc(
            &hmac_key,
            &package_key,
            &initialization_vector(&key_data.salt, Some(&HMAC_KEY_BLOCK_KEY), key_data.hash),
        );
        let encrypted_hmac_value = encrypt_aes_cbc(
            &zero_pad(hmac_value),
            &package_key,
            &initialization_vector(&key_data.salt, Some(&HMAC_VALUE_BLOCK_KEY), key_data.hash),
        );
        let xml = descriptor_xml(
            "e",
            "p",
            &key_data,
            &password_parameters,
            spin_count,
            &encrypted_hmac_key,
            &encrypted_hmac_value,
            &encrypted_verifier_input,
            &encrypted_verifier_hash,
            &encrypted_package_key,
        );
        let mut info = Vec::new();
        info.extend_from_slice(&AGILE_MAJOR_VERSION.to_le_bytes());
        info.extend_from_slice(&AGILE_MINOR_VERSION.to_le_bytes());
        info.extend_from_slice(&AGILE_RESERVED.to_le_bytes());
        info.extend_from_slice(xml.as_bytes());

        let cursor = Cursor::new(Vec::new());
        let mut compound = cfb::CompoundFile::create(cursor).unwrap();
        compound
            .create_stream("/EncryptionInfo")
            .unwrap()
            .write_all(&info)
            .unwrap();
        compound
            .create_stream("/EncryptedPackage")
            .unwrap()
            .write_all(&encrypted_package)
            .unwrap();
        compound.flush().unwrap();
        compound.into_inner().into_inner()
    }

    fn test_descriptor_xml(
        encryption_prefix: &str,
        password_prefix: &str,
        hash_name: &str,
        hash_size: usize,
        key_bits: u16,
        block_size: usize,
    ) -> String {
        let hash = match hash_name {
            "SHA1" | "SHA-1" => HashAlgorithm::Sha1,
            "SHA256" | "SHA-256" => HashAlgorithm::Sha256,
            "SHA384" | "SHA-384" => HashAlgorithm::Sha384,
            _ => HashAlgorithm::Sha512,
        };
        let key_data = CipherParameters {
            salt: vec![1; 16],
            key_bits,
            hash,
        };
        let password = CipherParameters {
            salt: vec![2; 16],
            key_bits,
            hash,
        };
        descriptor_xml_with_names(
            encryption_prefix,
            password_prefix,
            hash_name,
            hash_size,
            block_size,
            &key_data,
            &password,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn descriptor_xml(
        encryption_prefix: &str,
        password_prefix: &str,
        key_data: &CipherParameters,
        password: &CipherParameters,
        spin_count: u32,
        encrypted_hmac_key: &[u8],
        encrypted_hmac_value: &[u8],
        encrypted_verifier_input: &[u8],
        encrypted_verifier_hash: &[u8],
        encrypted_package_key: &[u8],
    ) -> String {
        format!(
            r#"<{e}:encryption xmlns:{e}="{encryption_ns}" xmlns:{p}="{password_ns}"><{e}:keyData saltSize="{key_salt_size}" blockSize="16" keyBits="{key_bits}" hashSize="{hash_size}" cipherAlgorithm="AES" cipherChaining="ChainingModeCBC" hashAlgorithm="{key_hash}" saltValue="{key_salt}"/><{e}:dataIntegrity encryptedHmacKey="{hmac_key}" encryptedHmacValue="{hmac_value}"/><{e}:keyEncryptors><{e}:keyEncryptor uri="{password_uri}"><{p}:encryptedKey spinCount="{spin_count}" saltSize="{password_salt_size}" blockSize="16" keyBits="{password_bits}" hashSize="{password_hash_size}" cipherAlgorithm="AES" cipherChaining="ChainingModeCBC" hashAlgorithm="{password_hash}" saltValue="{password_salt}" encryptedVerifierHashInput="{verifier_input}" encryptedVerifierHashValue="{verifier_hash}" encryptedKeyValue="{package_key}"/></{e}:keyEncryptor></{e}:keyEncryptors></{e}:encryption>"#,
            e = encryption_prefix,
            p = password_prefix,
            encryption_ns = String::from_utf8_lossy(ENCRYPTION_NS),
            password_ns = String::from_utf8_lossy(PASSWORD_NS),
            key_salt_size = key_data.salt.len(),
            key_bits = key_data.key_bits,
            hash_size = key_data.hash.output_size(),
            key_hash = key_data.hash.name(),
            key_salt = BASE64_STANDARD.encode(&key_data.salt),
            hmac_key = BASE64_STANDARD.encode(encrypted_hmac_key),
            hmac_value = BASE64_STANDARD.encode(encrypted_hmac_value),
            password_uri = PASSWORD_URI,
            spin_count = spin_count,
            password_salt_size = password.salt.len(),
            password_bits = password.key_bits,
            password_hash_size = password.hash.output_size(),
            password_hash = password.hash.name(),
            password_salt = BASE64_STANDARD.encode(&password.salt),
            verifier_input = BASE64_STANDARD.encode(encrypted_verifier_input),
            verifier_hash = BASE64_STANDARD.encode(encrypted_verifier_hash),
            package_key = BASE64_STANDARD.encode(encrypted_package_key),
        )
    }

    fn descriptor_xml_with_names(
        encryption_prefix: &str,
        password_prefix: &str,
        hash_name: &str,
        hash_size: usize,
        block_size: usize,
        key_data: &CipherParameters,
        password: &CipherParameters,
    ) -> String {
        format!(
            r#"<{e}:encryption xmlns:{e}="{encryption_ns}" xmlns:{p}="{password_ns}"><{e}:keyData saltSize="16" blockSize="{block_size}" keyBits="{key_bits}" hashSize="{hash_size}" cipherAlgorithm="AES" cipherChaining="ChainingModeCBC" hashAlgorithm="{hash_name}" saltValue="{key_salt}"/><{e}:dataIntegrity encryptedHmacKey="{hmac_key}" encryptedHmacValue="{hmac_value}"/><{e}:keyEncryptors><{e}:keyEncryptor uri="{password_uri}"><{p}:encryptedKey spinCount="1000" saltSize="16" blockSize="{block_size}" keyBits="{key_bits}" hashSize="{hash_size}" cipherAlgorithm="AES" cipherChaining="ChainingModeCBC" hashAlgorithm="{hash_name}" saltValue="{password_salt}" encryptedVerifierHashInput="{verifier_input}" encryptedVerifierHashValue="{verifier_hash}" encryptedKeyValue="{package_key}"/></{e}:keyEncryptor></{e}:keyEncryptors></{e}:encryption>"#,
            e = encryption_prefix,
            p = password_prefix,
            encryption_ns = String::from_utf8_lossy(ENCRYPTION_NS),
            password_ns = String::from_utf8_lossy(PASSWORD_NS),
            block_size = block_size,
            key_bits = key_data.key_bits,
            hash_size = hash_size,
            hash_name = hash_name,
            key_salt = BASE64_STANDARD.encode(&key_data.salt),
            hmac_key = BASE64_STANDARD.encode(vec![0; round_up(key_data.salt.len(), 16).unwrap()]),
            hmac_value = BASE64_STANDARD.encode(vec![0; round_up(hash_size, 16).unwrap()]),
            password_uri = PASSWORD_URI,
            password_salt = BASE64_STANDARD.encode(&password.salt),
            verifier_input = BASE64_STANDARD.encode(vec![0; 16]),
            verifier_hash = BASE64_STANDARD.encode(vec![0; round_up(hash_size, 16).unwrap()]),
            package_key =
                BASE64_STANDARD.encode(vec![0; round_up(key_data.key_bytes(), 16).unwrap()]),
        )
    }

    fn zero_pad(mut value: Vec<u8>) -> Vec<u8> {
        value.resize(round_up(value.len(), 16).unwrap(), 0);
        value
    }

    fn encrypt_aes_cbc(plaintext: &[u8], key: &[u8], iv: &[u8]) -> Vec<u8> {
        let mut buffer = plaintext.to_vec();
        macro_rules! encrypt {
            ($cipher:ty) => {
                cbc::Encryptor::<$cipher>::new_from_slices(key, iv)
                    .unwrap()
                    .encrypt_padded::<NoPadding>(&mut buffer, plaintext.len())
                    .unwrap()
            };
        }
        match key.len() {
            16 => {
                encrypt!(Aes128);
            }
            24 => {
                encrypt!(Aes192);
            }
            32 => {
                encrypt!(Aes256);
            }
            _ => panic!("unsupported test key size"),
        }
        buffer
    }
}
