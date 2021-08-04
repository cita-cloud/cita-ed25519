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

use super::{Address, PrivKey, PubKey};
use crate::error::Error;
use cita_crypto_trait::CreateKey;
use hashable::Hashable;
use rustc_serialize::hex::ToHex;
use sodiumoxide::{self,crypto::sign::gen_keypair};
use std::fmt;
use std::str::FromStr;

pub fn pubkey_to_address(pubkey: &PubKey) -> Address {

    println!("============ {:?}",pubkey);

    let x = pubkey.crypt_hash();
     println!("============ {:?}",x);
     let a = Address::from(x);
     println!("============ {:?}",a);
     a

    //Address::from()

    //Address::zero()
}

#[derive(Default)]
pub struct KeyPair {
    privkey: PrivKey,
    pubkey: PubKey,
}

impl fmt::Display for KeyPair {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        writeln!(f, "privkey:  {}", self.privkey.0.to_hex())?;
        writeln!(f, "pubkey:  {}", self.pubkey.0.to_hex())?;
        write!(f, "address:  {}", self.address().0.to_hex())
        
        
        //writeln!(f,"OKKKKKKKKKKKKKKKKKK")
    }
}

impl CreateKey for KeyPair {
    type PrivKey = PrivKey;
    type PubKey = PubKey;
    type Error = Error;
    type Address = Address;

    fn from_privkey(privkey: Self::PrivKey) -> Result<Self, Self::Error> {
        let pubkey = PubKey::from_slice(&privkey.0[32..]);
        Ok(KeyPair { privkey, pubkey })
    }

    fn gen_keypair() -> Self {
        //sodiumoxide::init();
        let (pk, sk) = gen_keypair();
        KeyPair {
            privkey: PrivKey::from(sk.0),
            pubkey: PubKey::from(pk.0),
        }
        // KeyPair {
        //     privkey: PrivKey::from_str("198cae5f9886c714c1badd74b17755804ad81f67d62aa90c711453582ecbadac198cae5f9886c714c1badd74b17755804ad81f67d62aa90c711453582ecbadac").unwrap(),
        //     pubkey:  PubKey::from_str("198cae5f9886c714c1badd74b17755804ad81f67d62aa90c711453582ecbadac").unwrap(), 
        // }
    }

    fn privkey(&self) -> &Self::PrivKey {
        &self.privkey
    }

    fn pubkey(&self) -> &Self::PubKey {
        &self.pubkey
    }

    fn address(&self) -> Address {
        pubkey_to_address(&self.pubkey)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cita_crypto_trait::CreateKey;

    // #[test]
    // fn test_from_privkey() {
    //     let keypair1 = KeyPair::gen_keypair();
    //     let keypair2 = KeyPair::from_privkey(keypair1.privkey).unwrap();
    //     assert_eq!(keypair1.pubkey, keypair2.pubkey);
    //     assert_eq!(keypair1.privkey, keypair2.privkey);
    // }
}
