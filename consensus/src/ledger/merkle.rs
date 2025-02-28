// Copyright (C) 2019-2021 Aleo Systems Inc.
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

use std::sync::Arc;

use crate::{IndexedDigests, Ledger};
use anyhow::*;
use indexmap::IndexSet;
use snarkos_storage::Digest;
use snarkvm_algorithms::MerkleParameters;

use super::indexed_merkle_tree::IndexedMerkleTree;

pub struct MerkleLedger<P: MerkleParameters + 'static> {
    ledger_digests: IndexSet<Digest>,
    commitments: IndexedMerkleTree<P>,
    serial_numbers: IndexedMerkleTree<P>,
    memos: IndexedDigests,
}

impl<P: MerkleParameters + 'static> MerkleLedger<P> {
    pub fn new(
        parameters: Arc<P>,
        ledger_digests: &[Digest],
        commitments: &[Digest],
        serial_numbers: &[Digest],
        memos: &[Digest],
    ) -> Result<Self> {
        Ok(Self {
            ledger_digests: ledger_digests.iter().cloned().collect(),
            commitments: IndexedMerkleTree::new(parameters.clone(), commitments)?,
            serial_numbers: IndexedMerkleTree::new(parameters, serial_numbers)?,
            memos: IndexedDigests::new(memos),
        })
    }
}

impl<P: MerkleParameters + 'static> Ledger for MerkleLedger<P> {
    fn extend(
        &mut self,
        new_commitments: &[Digest],
        new_serial_numbers: &[Digest],
        new_memos: &[Digest],
    ) -> Result<Digest> {
        self.commitments.extend(new_commitments)?;
        self.serial_numbers.extend(new_serial_numbers)?;
        self.memos.extend(new_memos);

        let new_digest = self.commitments.digest();
        self.ledger_digests.insert(new_digest.clone());

        Ok(new_digest)
    }

    fn push_interim_digests(&mut self, new_ledger_digests: &[Digest]) -> Result<()> {
        self.ledger_digests.extend(new_ledger_digests.iter().cloned());
        Ok(())
    }

    fn rollback(&mut self, commitments: &[Digest], serial_numbers: &[Digest], memos: &[Digest]) -> Result<()> {
        debug!(
            "rolling back merkle ledger: {} commitments, {} serial numbers, {} memos",
            commitments.len(),
            serial_numbers.len(),
            memos.len()
        );
        self.commitments.pop(commitments)?;
        self.serial_numbers.pop(serial_numbers)?;
        self.memos.pop(memos)?;

        let new_digest = self.commitments.digest();
        for i in (0..self.ledger_digests.len()).rev() {
            if self.ledger_digests[i] == new_digest {
                self.ledger_digests.truncate(i + 1);
                return Ok(());
            }
        }
        Err(anyhow!("couldn't find digest rollback point (partial rollback?)"))
    }

    fn clear(&mut self) {
        self.commitments.clear();
        self.serial_numbers.clear();
        self.memos.clear();
        self.ledger_digests.clear();
    }

    fn commitment_len(&self) -> usize {
        self.commitments.len()
    }

    fn contains_commitment(&self, commitment: &Digest) -> bool {
        self.commitments.contains(commitment)
    }

    fn commitment_index(&self, commitment: &Digest) -> Option<usize> {
        self.commitments.index(commitment)
    }

    fn contains_serial(&self, serial: &Digest) -> bool {
        self.serial_numbers.contains(serial)
    }

    fn contains_memo(&self, memo: &Digest) -> bool {
        self.memos.contains(memo)
    }

    fn validate_digest(&self, digest: &Digest) -> bool {
        self.ledger_digests.contains(digest)
    }

    fn digest(&self) -> Digest {
        self.ledger_digests
            .last()
            .cloned()
            .unwrap_or_else(|| self.commitments.digest()) // empty ledger
    }

    fn generate_proof(&self, commitment: &Digest, index: usize) -> Result<Vec<(Digest, Digest)>> {
        self.commitments.generate_proof(commitment, index)
    }

    fn validate_ledger(&self) -> bool {
        let calculated_digest = self.commitments.digest();
        self.digest() == calculated_digest
    }

    fn requires_async_task(&self, new_commitments_len: usize, new_serial_numbers_len: usize) -> bool {
        jumps_power_of_two(self.commitments.len(), new_commitments_len)
            || jumps_power_of_two(self.serial_numbers.len(), new_serial_numbers_len)
    }
}

fn jumps_power_of_two(start: usize, adding: usize) -> bool {
    let prior_depth = (start as f64).log2() as usize;
    let new_depth = ((start + adding) as f64).log2() as usize;
    prior_depth != new_depth
}
