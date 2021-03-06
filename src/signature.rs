// Copyright Rivtower Technologies LLC.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::{
    pubkey_to_address, Address, Error, KeyPair, Message, PrivKey, PubKey, SIGNATURE_BYTES_LEN,
};
use cita_crypto_trait::{CreateKey, Sign};
use rlp::*;
use rustc_serialize::hex::ToHex;
use serde::de::{Error as SerdeError, SeqAccess, Visitor};
use serde::ser::SerializeSeq;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sodiumoxide::crypto::sign::{
    sign_detached, verify_detached, PublicKey as EdPublicKey, SecretKey, Signature as EdSignature,
};

use std::fmt;
use std::ops::{Deref, DerefMut};

pub struct Signature(pub [u8; 96]);

impl Signature {
    pub fn sig(&self) -> &[u8] {
        &self.0[0..64]
    }

    pub fn pk(&self) -> &[u8] {
        &self.0[64..96]
    }
}

impl PartialEq for Signature {
    fn eq(&self, rhs: &Self) -> bool {
        self.0[..] == rhs.0[..]
    }
}

impl Decodable for Signature {
    fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
        rlp.decoder().decode_value(|bytes| {
            let mut sig = [0u8; 96];
            sig[0..96].copy_from_slice(bytes);
            Ok(Signature(sig))
        })
    }
}

impl Encodable for Signature {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.encoder().encode_value(&self.0[0..96]);
    }
}

// TODO: Maybe it should be implemented with rust macro
// https://github.com/rust-lang/rfcs/issues/1038
impl<'de> Deserialize<'de> for Signature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct SignatureVisitor;

        impl<'de> Visitor<'de> for SignatureVisitor {
            type Value = Signature;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("ed25519 signature")
            }

            fn visit_seq<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let mut signature = Signature([0u8; SIGNATURE_BYTES_LEN]);
                for i in 0..SIGNATURE_BYTES_LEN {
                    signature.0[i] = match visitor.next_element()? {
                        Some(val) => val,
                        None => return Err(SerdeError::invalid_length(SIGNATURE_BYTES_LEN, &self)),
                    }
                }
                Ok(signature)
            }
        }

        let visitor = SignatureVisitor;
        deserializer.deserialize_seq(visitor)
    }
}

impl Serialize for Signature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(SIGNATURE_BYTES_LEN))?;
        for i in 0..SIGNATURE_BYTES_LEN {
            seq.serialize_element(&self.0[i])?;
        }
        seq.end()
    }
}

impl Eq for Signature {}

impl fmt::Debug for Signature {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.debug_struct("Signature")
            .field("signature", &self.0[0..64].to_hex())
            .field("pubkey", &self.0[64..96].to_hex())
            .finish()
    }
}

impl fmt::Display for Signature {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.0[..].to_hex())
    }
}

impl Default for Signature {
    fn default() -> Self {
        Signature([0u8; 96])
    }
}

impl Clone for Signature {
    fn clone(&self) -> Self {
        Signature(self.0)
    }
}

impl From<[u8; 96]> for Signature {
    fn from(bytes: [u8; 96]) -> Self {
        Signature(bytes)
    }
}

impl Into<[u8; 96]> for Signature {
    fn into(self) -> [u8; 96] {
        self.0
    }
}

impl<'a> From<&'a [u8]> for Signature {
    fn from(slice: &'a [u8]) -> Signature {
        assert_eq!(slice.len(), SIGNATURE_BYTES_LEN);
        let mut bytes = [0u8; 96];
        bytes.copy_from_slice(slice);
        Signature(bytes)
    }
}

impl<'a> Into<&'a [u8]> for &'a Signature {
    fn into(self) -> &'a [u8] {
        &self.0[..]
    }
}

impl fmt::LowerHex for Signature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in &self.0[..] {
            write!(f, "{:02x}", i)?;
        }
        Ok(())
    }
}

impl From<Signature> for String {
    fn from(s: Signature) -> Self {
        format!("{:x}", s)
    }
}

impl Deref for Signature {
    type Target = [u8; 96];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Signature {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Sign for Signature {
    type PrivKey = PrivKey;
    type PubKey = PubKey;
    type Message = Message;
    type Error = Error;
    type Address = Address;

    fn sign(privkey: &Self::PrivKey, message: &Self::Message) -> Result<Self, Self::Error> {
        let keypair = KeyPair::from_privkey(*privkey)?;
        let secret_key = SecretKey::from_slice(privkey.as_ref()).unwrap();
        let pubkey = keypair.pubkey();
        let mut ret = [0u8; 96];
        let sig = sign_detached(message.as_ref(), &secret_key);

        ret[0..64].copy_from_slice(sig.as_ref());
        ret[64..96].copy_from_slice(pubkey.as_ref());
        Ok(Signature(ret))
    }

    fn recover(&self, message: &Self::Message) -> Result<Self::PubKey, Self::Error> {
        let sig = self.sig();
        let pubkey = self.pk();

        let mut sig_array = [0; 64];
        sig_array.copy_from_slice(sig);

        let is_valid = verify_detached(
            &EdSignature::new(sig_array),
            message.as_ref(),
            &EdPublicKey::from_slice(pubkey).unwrap(),
        );

        if !is_valid {
            Err(Error::InvalidSignature)
        } else {
            Ok(PubKey::from_slice(pubkey))
        }
    }

    fn verify_public(
        &self,
        pubkey: &Self::PubKey,
        message: &Self::Message,
    ) -> Result<bool, Self::Error> {
        let sig = self.sig();
        let pk = self.pk();

        let mut sig_array = [0; 64];
        sig_array.copy_from_slice(sig);

        if pk != pubkey.as_ref() as &[u8] {
            return Err(Error::InvalidPubKey);
        }

        let is_valid = verify_detached(
            &EdSignature::new(sig_array),
            message.as_ref(),
            &EdPublicKey::from_slice(pubkey.as_ref()).unwrap(),
        );
        if !is_valid {
            Err(Error::InvalidSignature)
        } else {
            Ok(true)
        }
    }

    fn verify_address(&self, address: &Address, message: &Message) -> Result<bool, Self::Error> {
        let pubkey = self.recover(message)?;
        let recover_address = pubkey_to_address(&pubkey);
        Ok(address == &recover_address)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bincode::{deserialize, serialize, Infinite};
    use cita_crypto_trait::CreateKey;

    const MESSAGE: [u8; 32] = [
        0x01, 0x02, 0x03, 0x04, 0x19, 0xab, 0xfe, 0x39, 0x6f, 0x28, 0x79, 0x00, 0x08, 0xdf, 0x9a,
        0xef, 0xfb, 0x77, 0x42, 0xae, 0xad, 0xfc, 0xcf, 0x12, 0x24, 0x45, 0x29, 0x89, 0x29, 0x45,
        0x3f, 0xf8,
    ];

    #[test]
    fn test_sign_verify() {
        let keypair = KeyPair::gen_keypair();
        let msg = Message::from_slice(&MESSAGE[..]);
        let sig = Signature::sign(keypair.privkey(), &msg).unwrap();
        assert!(sig.verify_public(keypair.pubkey(), &msg).unwrap());
    }

    #[test]
    fn test_verify_address() {
        let keypair = KeyPair::gen_keypair();
        let address = pubkey_to_address(keypair.pubkey());
        let msg = Message::from_slice(&MESSAGE[..]);
        let sig = Signature::sign(keypair.privkey(), &msg).unwrap();
        assert!(sig.verify_address(&address, &msg).unwrap());
    }

    #[test]
    fn test_recover() {
        let keypair = KeyPair::gen_keypair();
        let msg = Message::from_slice(&MESSAGE[..]);
        let sig = Signature::sign(keypair.privkey(), &msg).unwrap();
        assert_eq!(keypair.pubkey(), &sig.recover(&msg).unwrap());
    }

    #[test]
    fn test_into_slice() {
        let keypair = KeyPair::gen_keypair();
        let msg = Message::from_slice(&MESSAGE[..]);
        let sig = Signature::sign(keypair.privkey(), &msg).unwrap();
        let sig = &sig;
        let slice: &[u8] = sig.into();
        assert_eq!(Signature::from(slice), *sig);
    }

    #[test]
    fn test_de_serialize() {
        let keypair = KeyPair::gen_keypair();
        let msg = Message::from_slice(&MESSAGE[..]);
        let sig = Signature::sign(keypair.privkey(), &msg).unwrap();
        let se_result = serialize(&sig, Infinite).unwrap();
        let de_result: Signature = deserialize(&se_result).unwrap();
        assert_eq!(sig, de_result);
    }
}
