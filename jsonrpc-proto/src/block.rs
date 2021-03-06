// CITA
// Copyright 2016-2018 Cryptape Technologies LLC.

// This program is free software: you can redistribute it
// and/or modify it under the terms of the GNU General Public
// License as published by the Free Software Foundation,
// either version 3 of the License, or (at your option) any
// later version.

// This program is distributed in the hope that it will be
// useful, but WITHOUT ANY WARRANTY; without even the implied
// warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR
// PURPOSE. See the GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use cita_types::{Address, H256, U256};
use jsonrpc_types::rpc_types::{
    Block, BlockBody, BlockHeader, BlockTransaction, FullTransaction, Proof, RpcBlock,
};
use jsonrpc_types::Error;

use crate::from_into::TryFromProto;

pub trait BlockExt {
    type Error;

    fn try_from_rpc_block(rpc_block: RpcBlock) -> Result<Block, Self::Error>;
}

impl TryFromProto<libproto::BlockHeader> for BlockHeader {
    type Error = Error;

    fn try_from_proto(proto_header: libproto::BlockHeader) -> Result<Self, Self::Error> {
        let proof: Option<Proof> = match proto_header.get_height() {
            0 | 1 => None,
            _ => Some(Proof::try_from_proto(proto_header.clone().take_proof())?),
        };
        trace!(
            "number = {}, proof = {:?}",
            U256::from(proto_header.get_height()),
            proof
        );

        Ok(BlockHeader {
            timestamp: proto_header.timestamp,
            prev_hash: H256::from(proto_header.get_prevhash()),
            number: U256::from(proto_header.get_height()),
            state_root: H256::from(proto_header.get_state_root()),
            transactions_root: H256::from(proto_header.get_transactions_root()),
            receipts_root: H256::from(proto_header.get_receipts_root()),
            quota_used: U256::from(proto_header.get_quota_used()),
            proof,
            proposer: Address::from(proto_header.get_proposer()),
        })
    }
}

impl BlockExt for Block {
    type Error = Error;

    fn try_from_rpc_block(rpc_block: RpcBlock) -> Result<Self, Self::Error> {
        use crate::error::ErrorExt;
        use libproto::TryFrom;

        let mut blk = libproto::Block::try_from(&rpc_block.block) // from chain
            .map_err(|err| Error::rpc_block_decode_error(Box::new(err)))?;

        let block_transactions = blk.take_body().take_transactions();
        let transactions = if rpc_block.include_txs {
            block_transactions
                .into_iter()
                .map(|x| FullTransaction::try_from_proto(x).map(BlockTransaction::Full))
                .collect::<Result<Vec<BlockTransaction>, Error>>()?
        } else {
            block_transactions
                .into_iter()
                .map(|x| BlockTransaction::Hash(H256::from_slice(x.get_tx_hash())))
                .collect()
        };
        let header = BlockHeader::try_from_proto(blk.take_header())?;

        Ok(Block {
            version: blk.version,
            header,
            body: BlockBody { transactions },
            hash: H256::from_slice(&rpc_block.hash),
        })
    }
}
