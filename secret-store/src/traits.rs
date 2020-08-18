// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of Open Ethereum.

// Open Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Open Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Open Ethereum.  If not, see <http://www.gnu.org/licenses/>.

use ethereum_types::{Address, H256};
use ethkey::{Error as EthKeyError, KeyPair, Signature};
use std::collections::BTreeSet;
use types::{
    EncryptedDocumentKey, EncryptedDocumentKeyShadow, EncryptedMessageSignature, Error,
    MessageHash, NodeId, Public, RequestSignature, Requester, ServerKeyId,
};

/// Node key pair.
pub trait NodeKeyPair: Send + Sync {
    /// Public portion of key.
    fn public(&self) -> &Public;
    /// Address of key owner.
    fn address(&self) -> Address;
    /// Sign data with node key.
    fn sign(&self, data: &H256) -> Result<Signature, EthKeyError>;
    /// Compute shared key to encrypt channel between two nodes.
    fn compute_shared_key(&self, peer_public: &Public) -> Result<KeyPair, EthKeyError>;
}

/// Server key (SK) generator.
pub trait ServerKeyGenerator {
    /// Generate new SK.
    /// `key_id` is the caller-provided identifier of generated SK.
    /// `author` is the author of key entry.
    /// `threshold + 1` is the minimal number of nodes, required to restore private key.
    /// Result is a public portion of SK.
    fn generate_key(
        &self,
        key_id: &ServerKeyId,
        author: &Requester,
        threshold: usize,
    ) -> Result<Public, Error>;
    /// Retrieve public portion of previously generated SK.
    /// `key_id` is identifier of previously generated SK.
    /// `author` is the same author, that has created the server key.
    fn restore_key_public(&self, key_id: &ServerKeyId, author: &Requester)
        -> Result<Public, Error>;
}

/// Document key (DK) server.
pub trait DocumentKeyServer: ServerKeyGenerator {
    /// Store externally generated DK.
    /// `key_id` is identifier of previously generated SK.
    /// `author` is the same author, that has created the server key.
    /// `common_point` is a result of `k * T` expression, where `T` is generation point and `k` is random scalar in EC field.
    /// `encrypted_document_key` is a result of `M + k * y` expression, where `M` is unencrypted document key (point on EC),
    ///   `k` is the same scalar used in `common_point` calculation and `y` is previously generated public part of SK.
    fn store_document_key(
        &self,
        key_id: &ServerKeyId,
        author: &Requester,
        common_point: Public,
        encrypted_document_key: Public,
    ) -> Result<(), Error>;
    /// Generate and store both SK and DK. This is a shortcut for consequent calls of `generate_key` and `store_document_key`.
    /// The only difference is that DK is generated by DocumentKeyServer (which might be considered unsafe).
    /// `key_id` is the caller-provided identifier of generated SK.
    /// `author` is the author of server && document key entry.
    /// `threshold + 1` is the minimal number of nodes, required to restore private key.
    /// Result is a DK, encrypted with caller public key.
    fn generate_document_key(
        &self,
        key_id: &ServerKeyId,
        author: &Requester,
        threshold: usize,
    ) -> Result<EncryptedDocumentKey, Error>;
    /// Restore previously stored DK.
    /// DK is decrypted on the key server (which might be considered unsafe), and then encrypted with caller public key.
    /// `key_id` is identifier of previously generated SK.
    /// `requester` is the one who requests access to document key. Caller must be on ACL for this function to succeed.
    /// Result is a DK, encrypted with caller public key.
    fn restore_document_key(
        &self,
        key_id: &ServerKeyId,
        requester: &Requester,
    ) -> Result<EncryptedDocumentKey, Error>;
    /// Restore previously stored DK.
    /// To decrypt DK on client:
    /// 1) use requestor secret key to decrypt secret coefficients from result.decrypt_shadows
    /// 2) calculate decrypt_shadows_sum = sum of all secrets from (1)
    /// 3) calculate decrypt_shadow_point: decrypt_shadows_sum * result.common_point
    /// 4) calculate decrypted_secret: result.decrypted_secret + decrypt_shadow_point
    /// Result is a DK shadow.
    fn restore_document_key_shadow(
        &self,
        key_id: &ServerKeyId,
        requester: &Requester,
    ) -> Result<EncryptedDocumentKeyShadow, Error>;
}

/// Message signer.
pub trait MessageSigner: ServerKeyGenerator {
    /// Generate Schnorr signature for message with previously generated SK.
    /// `key_id` is the caller-provided identifier of generated SK.
    /// `requester` is the one who requests access to server key private.
    /// `message` is the message to be signed.
    /// Result is a signed message, encrypted with caller public key.
    fn sign_message_schnorr(
        &self,
        key_id: &ServerKeyId,
        requester: &Requester,
        message: MessageHash,
    ) -> Result<EncryptedMessageSignature, Error>;
    /// Generate ECDSA signature for message with previously generated SK.
    /// WARNING: only possible when SK was generated using t <= 2 * N.
    /// `key_id` is the caller-provided identifier of generated SK.
    /// `signature` is `key_id`, signed with caller public key.
    /// `message` is the message to be signed.
    /// Result is a signed message, encrypted with caller public key.
    fn sign_message_ecdsa(
        &self,
        key_id: &ServerKeyId,
        signature: &Requester,
        message: MessageHash,
    ) -> Result<EncryptedMessageSignature, Error>;
}

/// Administrative sessions server.
pub trait AdminSessionsServer {
    /// Change servers set so that nodes in new_servers_set became owners of shares for all keys.
    /// And old nodes (i.e. cluster nodes except new_servers_set) have clear databases.
    /// WARNING: newly generated keys will be distributed among all cluster nodes. So this session
    /// must be followed with cluster nodes change (either via contract, or config files).
    fn change_servers_set(
        &self,
        old_set_signature: RequestSignature,
        new_set_signature: RequestSignature,
        new_servers_set: BTreeSet<NodeId>,
    ) -> Result<(), Error>;
}

/// Key server.
pub trait KeyServer: AdminSessionsServer + DocumentKeyServer + MessageSigner + Send + Sync {}
