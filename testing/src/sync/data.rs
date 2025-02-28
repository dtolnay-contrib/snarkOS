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

use snarkos_parameters::GenesisBlock;
use snarkos_storage::{Digest, SerialBlock, SerialBlockHeader, SerialRecord, SerialTransaction, VMBlock, VMRecord};
use snarkvm_dpc::{
    testnet1::{instantiated::*, record::Record as DPCRecord},
    Block,
};
use snarkvm_parameters::traits::genesis::Genesis;
use snarkvm_utilities::bytes::{FromBytes, ToBytes};

use once_cell::sync::Lazy;
use std::io::{Read, Result as IoResult, Write};

/// Helper providing pre-calculated data for e2e tests
pub static DATA: Lazy<TestData> = Lazy::new(load_test_data);

pub static GENESIS_BLOCK_HEADER_HASH: Lazy<[u8; 32]> = Lazy::new(|| genesis().header.get_hash().0);

pub static BLOCK_1: Lazy<SerialBlock> = Lazy::new(|| DATA.block_1.clone());
pub static BLOCK_1_HEADER_HASH: Lazy<Digest> = Lazy::new(|| DATA.block_1.header.hash());

pub static BLOCK_2: Lazy<SerialBlock> = Lazy::new(|| DATA.block_2.clone());
pub static BLOCK_2_HEADER_HASH: Lazy<Digest> = Lazy::new(|| DATA.block_2.header.hash());

pub static TRANSACTION_1: Lazy<SerialTransaction> = Lazy::new(|| DATA.block_1.transactions[0].clone());
pub static TRANSACTION_2: Lazy<SerialTransaction> = Lazy::new(|| DATA.block_2.transactions[0].clone());

// Alternative blocks used for testing syncs and rollbacks
pub static ALTERNATIVE_BLOCK_1: Lazy<SerialBlock> = Lazy::new(|| SerialBlock {
    header: DATA.alternative_block_1_header.clone(),
    transactions: DATA.block_1.transactions.clone(),
});

pub static ALTERNATIVE_BLOCK_2: Lazy<SerialBlock> = Lazy::new(|| SerialBlock {
    header: DATA.alternative_block_2_header.clone(),
    transactions: DATA.block_2.transactions.clone(),
});

pub fn genesis() -> Block<Testnet1Transaction> {
    FromBytes::read_le(GenesisBlock::load_bytes().as_slice()).unwrap()
}

pub struct TestData {
    pub block_1: SerialBlock,
    pub block_2: SerialBlock,
    pub records_1: Vec<SerialRecord>,
    pub records_2: Vec<SerialRecord>,
    pub alternative_block_1_header: SerialBlockHeader,
    pub alternative_block_2_header: SerialBlockHeader,
}

impl ToBytes for TestData {
    #[inline]
    fn write_le<W: Write>(&self, mut writer: W) -> IoResult<()> {
        self.block_1.write_le(&mut writer)?;

        self.block_2.write_le(&mut writer)?;

        writer.write_all(&(self.records_1.len() as u64).to_le_bytes())?;
        self.records_1.write_le(&mut writer)?;

        writer.write_all(&(self.records_2.len() as u64).to_le_bytes())?;
        self.records_2.write_le(&mut writer)?;

        self.alternative_block_1_header.write_le(&mut writer)?;
        self.alternative_block_2_header.write_le(&mut writer)?;

        Ok(())
    }
}

impl FromBytes for TestData {
    fn read_le<R: Read>(mut reader: R) -> IoResult<Self> {
        let block_1: Block<Testnet1Transaction> = FromBytes::read_le(&mut reader)?;

        let block_2: Block<Testnet1Transaction> = FromBytes::read_le(&mut reader)?;

        let len = u64::read_le(&mut reader)? as usize;
        let records_1 = (0..len)
            .map(|_| -> DPCRecord<Components> { FromBytes::read_le(&mut reader).unwrap() })
            .map(|x| x.serialize().unwrap())
            .collect::<Vec<_>>();

        let len = u64::read_le(&mut reader)? as usize;
        let records_2 = (0..len)
            .map(|_| -> DPCRecord<Components> { FromBytes::read_le(&mut reader).unwrap() })
            .map(|x| x.serialize().unwrap())
            .collect::<Vec<_>>();

        let alternative_block_1_header: SerialBlockHeader = FromBytes::read_le(&mut reader)?;
        let alternative_block_2_header: SerialBlockHeader = FromBytes::read_le(&mut reader)?;

        Ok(Self {
            block_1: <_ as VMBlock>::serialize(&block_1).unwrap(),
            block_2: <_ as VMBlock>::serialize(&block_2).unwrap(),
            records_1,
            records_2,
            alternative_block_1_header,
            alternative_block_2_header,
        })
    }
}

fn load_test_data() -> TestData {
    TestData::read_le(&include_bytes!("test_data")[..]).unwrap()
}

#[derive(Debug)]
pub struct TestBlocks(pub Vec<SerialBlock>);

impl TestBlocks {
    pub fn new(blocks: Vec<SerialBlock>) -> Self {
        TestBlocks(blocks)
    }

    pub fn load(count: Option<usize>, batch_name: &str) -> Self {
        let blocks_path = format!("{}/src/sync/{}", env!("CARGO_MANIFEST_DIR"), batch_name);
        let blocks_bytes = std::fs::read(&blocks_path).unwrap();
        TestBlocks::read_le(&*blocks_bytes, count).unwrap()
    }

    pub fn write_le<W: Write>(&self, mut writer: W) -> IoResult<()> {
        for block in &self.0 {
            // Clone is necessary here, otherwise weird things happen.
            let block = block.clone();
            block.write_le(&mut writer)?;
        }

        Ok(())
    }

    pub fn read_le<R: Read>(mut reader: R, count: Option<usize>) -> IoResult<Self> {
        let mut blocks = Vec::new();

        if let Some(count) = count {
            blocks.reserve(count);

            for _ in 0..count {
                let block: Block<Testnet1Transaction> = FromBytes::read_le(&mut reader)?;
                blocks.push(<Block<Testnet1Transaction> as VMBlock>::serialize(&block).unwrap());
            }
        } else {
            while let Ok(block) = FromBytes::read_le(&mut reader) {
                blocks.push(<Block<Testnet1Transaction> as VMBlock>::serialize(&block).unwrap());
            }
        }

        Ok(TestBlocks::new(blocks))
    }
}
