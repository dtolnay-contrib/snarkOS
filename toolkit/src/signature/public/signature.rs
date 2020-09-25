// Copyright (C) 2019-2020 Aleo Systems Inc.
// This file is part of the snarkOS library.

// The snarkOS library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The snarkOS library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the snarkOS library. If not, see <https://www.gnu.org/licenses/>.

use crate::{
    account::{Address, ViewKey},
    errors::SignatureError,
};

use snarkos_dpc::base_dpc::{instantiated::Components, parameters::SystemParameters};
use snarkos_models::{algorithms::SignatureScheme, dpc::DPCComponents};
use snarkos_utilities::{
    bytes::{FromBytes, ToBytes},
    to_bytes,
};

use rand::{CryptoRng, Rng};
use std::{fmt, str::FromStr};

/// Schnorr signature scheme where:
///   The Account View Key is the signature private key.
///   The Account Address is the signature public key.
#[derive(Debug)]
pub struct Signature {
    pub(crate) signature: <<Components as DPCComponents>::AccountEncryption as SignatureScheme>::Output,
}

impl Signature {
    pub fn sign<R: Rng + CryptoRng>(view_key: &ViewKey, message: &[u8], rng: &mut R) -> Result<Self, SignatureError> {
        let parameters = SystemParameters::<Components>::load()?;

        let signature = parameters
            .account_encryption
            .sign(&view_key.view_key.decryption_key, message, rng)?;

        Ok(Self { signature })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut output = vec![];
        self.signature
            .write(&mut output)
            .expect("serialization to bytes failed");
        output
    }

    pub fn verify(&self, address: &Address, message: &[u8]) -> Result<bool, SignatureError> {
        let parameters = SystemParameters::<Components>::load()?;

        Ok(parameters
            .account_encryption
            .verify(&address.address.encryption_key, message, &self.signature)?)
    }
}

impl FromStr for Signature {
    type Err = SignatureError;

    fn from_str(signature: &str) -> Result<Self, Self::Err> {
        let signature_bytes = hex::decode(signature)?;
        let signature: <<Components as DPCComponents>::AccountEncryption as SignatureScheme>::Output =
            FromBytes::read(&signature_bytes[..])?;

        Ok(Self { signature })
    }
}

impl fmt::Display for Signature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            hex::encode(to_bytes![self.signature].expect("failed to convert to bytes"))
        )
    }
}
