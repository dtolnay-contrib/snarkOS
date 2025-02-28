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

use snarkos_metrics as metrics;

use std::time::Instant;

use chrono::{DateTime, Utc};

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct SyncState {
    /// number of requested sync blocks
    pub total_sync_blocks: u32,
    /// The number of remaining blocks to sync with.
    pub remaining_sync_blocks: u32,

    /// The number of sync blocks sent to this peer.
    pub blocks_synced_to: u32,
    /// The number of sync blocks received from this peer.
    pub blocks_synced_from: u32,
    /// The number of blocks received from this peer.
    pub blocks_received_from: u32,
    /// The number of blocks sent to this peer.
    pub blocks_sent_to: u32,
}

impl SyncState {
    pub fn sync_from_storage(&mut self, data: &snarkos_storage::Peer) {
        self.blocks_synced_to = data.blocks_synced_to;
        self.blocks_synced_from = data.blocks_synced_from;
        self.blocks_received_from = data.blocks_received_from;
        self.blocks_sent_to = data.blocks_sent_to;
    }

    pub fn reset(&mut self) {
        self.remaining_sync_blocks = 0;
        self.total_sync_blocks = 0;
    }
}

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct PeerQuality {
    /// The number of failures associated with the peer; grounds for dismissal.
    pub failures: Vec<DateTime<Utc>>,

    /// The last time the node interacted with this peer.
    pub last_seen: Option<DateTime<Utc>>,
    /// The first time the node interacted with this peer.
    pub first_seen: Option<DateTime<Utc>>,
    /// The last time the node was connected to this peer.
    pub last_connected: Option<DateTime<Utc>>,
    /// The last time the node was disconnected from this peer.
    pub last_disconnected: Option<DateTime<Utc>>,

    /// The number of times we have attempted to connect to this peer.
    pub connection_attempt_count: u64,
    /// The number of failed connection attempts since the last connection success
    pub connection_transient_fail_count: u64,
    /// The number of times we have connected to this peer.
    pub connected_count: u64,
    /// The number of times we have disconnected from this peer.
    pub disconnected_count: u64,

    /// Set to `true` if this node has sent a `Ping` and is expecting a `Pong` in return.
    #[serde(skip)]
    pub expecting_pong: bool,
    /// The timestamp of the last sent `Ping` to this peer.
    #[serde(skip)]
    pub last_ping_sent: Option<Instant>,
    /// The time it took to send a `Ping` to the peer and for it to respond with a `Pong`.
    pub rtt_ms: u64,

    /// The number of messages received from this peer.
    pub num_messages_received: u64,
}

impl PeerQuality {
    pub fn sync_from_storage(&mut self, peer: &snarkos_storage::Peer) {
        self.last_seen = peer.last_seen;
        self.first_seen = peer.first_seen;
        self.last_connected = peer.last_connected;

        self.connection_attempt_count = peer.connection_attempt_count;
        self.connection_transient_fail_count = peer.connection_transient_fail_count;
        self.connected_count = peer.connection_success_count;
    }

    pub fn is_inactive(&self, now: DateTime<Utc>) -> bool {
        let last_seen = self.last_seen;
        if let Some(last_seen) = last_seen {
            now - last_seen > chrono::Duration::seconds(crate::MAX_PEER_INACTIVITY_SECS.into())
        } else {
            // in the peer book, but never been connected to before
            false
        }
    }

    pub fn see(&mut self) {
        let now = chrono::Utc::now();
        if self.first_seen.is_none() {
            self.first_seen = Some(now);
        }
        self.last_seen = Some(now);
    }

    pub fn connected(&mut self) {
        self.see();
        self.connection_transient_fail_count = 0;
        self.last_connected = Some(chrono::Utc::now());
        self.connected_count += 1;
    }

    pub fn connecting(&mut self) {
        self.connection_attempt_count += 1;
    }

    pub fn connect_failed(&mut self) {
        self.connection_transient_fail_count += 1;
    }

    pub fn disconnected(&mut self) {
        let disconnect_timestamp = chrono::Utc::now();

        self.see();
        self.last_disconnected = Some(disconnect_timestamp);
        self.disconnected_count += 1;
        self.expecting_pong = false;

        if let Some(last_connected) = self.last_connected {
            if let Ok(elapsed) = disconnect_timestamp.signed_duration_since(last_connected).to_std() {
                metrics::histogram!(metrics::connections::DURATION, elapsed);
            }
        }
    }
}
