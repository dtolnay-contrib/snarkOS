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

use std::net::SocketAddr;

use crate::{Peer, PeerHandle};

#[derive(Debug)]
pub enum PeerEventData {
    Connected(PeerHandle),
    Disconnect(Box<Peer>),
    FailHandshake,
}

#[derive(Debug)]
pub struct PeerEvent {
    pub address: SocketAddr,
    pub data: PeerEventData,
}
