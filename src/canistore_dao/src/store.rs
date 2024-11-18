use candid::{CandidType, Decode, Encode, Principal};
use canistore_types::{
    certificate::{CertificateInfo, MusicCertificate, MusicCopyright},
    dao::{CanisterDeploy, DaoStateInfo},
};
use ciborium::{from_reader, into_writer};
use ic_cdk::storage;
use ic_certified_map::{leaf_hash, AsHashTree, RbTree};
use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    storable::Bound,
    DefaultMemoryImpl, StableBTreeMap, StableCell, Storable,
};
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    cell::{Cell, RefCell},
    collections::HashMap,
    mem,
};

type Memory = VirtualMemory<DefaultMemoryImpl>;

#[derive(CandidType, Clone, Deserialize, Serialize, Debug)]
pub struct State {
    pub name: String,
    pub owner: Principal,
    pub user_canister_id: Principal,
    pub platform_canister_id: Principal,
    pub user_space_infos: HashMap<Principal, Vec<Principal>>,
    pub is_open: bool,
    pub canister_list: Vec<CanisterDeploy>,
}

impl State {
    pub fn to_state_info(&self) -> DaoStateInfo {
        DaoStateInfo {
            name: self.name.clone(),
            user_canister_id: self.user_canister_id,
            platform_canister_id: self.platform_canister_id,
            is_open: self.is_open,
            sub_canisters: self.canister_list.clone(),
        }
    }
}

#[derive(CandidType, Deserialize)]
pub struct StorableCert {
    pub state: State,
    pub cert_counter: u128,
    pub cert: Vec<(String, Vec<u8>)>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            name: String::from("Canistore DAO"),
            owner: Principal::anonymous(),
            user_canister_id: Principal::anonymous(),
            platform_canister_id: Principal::anonymous(),
            user_space_infos: HashMap::new(),
            is_open: true,
            canister_list: vec![],
        }
    }
}

impl Storable for State {
    const BOUND: Bound = Bound::Unbounded;

    fn to_bytes(&self) -> Cow<[u8]> {
        let mut buf = vec![];
        into_writer(self, &mut buf).expect("failed to encode User DAO data");
        Cow::Owned(buf)
    }

    fn from_bytes(bytes: Cow<'_, [u8]>) -> Self {
        from_reader(&bytes[..]).expect("failed to decode User DAO data")
    }
}

#[derive(CandidType, Clone, Deserialize, Debug)]
pub struct CertificateInfoWrapper(pub CertificateInfo);

impl Storable for CertificateInfoWrapper {
    const BOUND: Bound = Bound::Unbounded;

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(Encode!(self).unwrap())
    }
}

impl CertificateInfoWrapper {
    pub fn into_inner(self) -> CertificateInfo {
        self.0
    }
}

const STATE_MEMORY_ID: MemoryId = MemoryId::new(0);
const CERTIFIED_MEMORY_ID: MemoryId = MemoryId::new(1);

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());

    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static STATE_STORE: RefCell<StableCell<State, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with_borrow(|m| m.get(STATE_MEMORY_ID)),
            State::default()
        ).expect("failed to init STATE_STORE")
    );

    static CERTIFIED_STORE: RefCell<StableBTreeMap<String, CertificateInfoWrapper, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with_borrow(|m| m.get(CERTIFIED_MEMORY_ID)),
        )
    );

    static CERT_COUNTER: Cell<u128> = Cell::new(0);
    static CERTIFIED_TREE: RefCell<RbTree<String, Vec<u8>>> = RefCell::new(RbTree::new());
}

pub mod state {
    use ic_cdk::api::time;
    use serde_bytes::ByteArray;

    use super::*;

    pub fn with<R>(f: impl FnOnce(&State) -> R) -> R {
        STATE.with(|r| f(&r.borrow()))
    }

    pub fn with_mut<R>(f: impl FnOnce(&mut State) -> R) -> R {
        STATE.with(|r| f(&mut r.borrow_mut()))
    }

    pub fn save() {
        STATE.with(|h| {
            STATE_STORE.with(|r| {
                r.borrow_mut()
                    .set(h.borrow().clone())
                    .expect("failed to save User DAO data");
            });
        });
    }

    pub fn is_open() -> bool {
        state::with(|state| state.is_open)
    }

    pub fn add_canister(canister_id: Principal, wasm_name: String, hash: ByteArray<32>) {
        STATE.with(|r| {
            let mut state = r.borrow_mut();
            state.canister_list.push(CanisterDeploy {
                deploy_at: time(),
                canister: canister_id,
                wasm_name,
                wasm_hash: hash,
            });
        });
    }
}

pub mod certified {
    use crate::CERTIFIED_HEADER;
    use canistore_types::{
        certificate::MusicCertificateResp,
        error::{CustomError, ErrorCode},
    };

    use super::*;

    pub fn store_certificate(
        certificate: MusicCertificate,
    ) -> Result<MusicCertificateResp, String> {
        let count = CERT_COUNTER.with(|counter| {
            let count = counter.get() + 1;
            counter.set(count);
            count
        });

        let serialized_cert = candid::encode_one(&certificate)
            .map_err(|e| format!("Failed to encode certificate: {}", e))?;

        let hash = leaf_hash(&serialized_cert);
        let hash_hex = hex::encode(&hash);
        let key = format!("{}_{}", CERTIFIED_HEADER, count);

        CERTIFIED_TREE.with(|tree| {
            let mut tree = tree.borrow_mut();
            tree.insert(key.clone(), hash.to_vec());
            ic_cdk::api::set_certified_data(&tree.root_hash());
        });

        // Store certificate info in the CERTIFIED_STORE
        cert::add_cret_info(
            key.clone(),
            CertificateInfo {
                key: key.clone(),
                cert_info: certificate,
                cert_hex: hash_hex.clone(),
            },
        );

        Ok(MusicCertificateResp {
            key,
            music_cert_hex: hash_hex,
        })
    }

    pub fn get_certificate(key: String) -> Result<MusicCopyright, String> {
        // Retrieve the data certificate from IC
        let certificate = ic_cdk::api::data_certificate()
            .ok_or_else(|| CustomError::new(ErrorCode::NoDataFound, None).to_string())?;

        // Retrieve the witness from the CERTIFIED_TREE
        let witness = CERTIFIED_TREE.with(|tree| {
            let tree = tree.borrow();
            let mut witness = vec![];
            let mut witness_serializer = serde_cbor::Serializer::new(&mut witness);

            if let Err(e) = witness_serializer.self_describe() {
                return Err(format!("Failed to self describe witness: {:?}", e));
            }

            tree.witness(key.as_bytes())
                .serialize(&mut witness_serializer)
                .map_err(|_| CustomError::new(ErrorCode::NoDataFound, None).to_string())?;
            Ok(witness)
        })?;

        // Retrieve the count value
        let count = CERT_COUNTER.with(|counter| counter.get());

        // Retrieve the serialized certificate from the CERTIFIED_TREE
        let serialized_cert = CERTIFIED_TREE.with(|tree| {
            let tree = tree.borrow();
            tree.get(key.as_bytes())
                .ok_or_else(|| CustomError::new(ErrorCode::NoDataFound, None).to_string())
                .map(|cert| cert.clone())
        })?;

        let music_cert_hex = hex::encode(&serialized_cert);

        // Return the MusicCopyright structure as a result
        Ok(MusicCopyright {
            count,
            certificate,
            music_cert_hex,
            witness,
        })
    }

    pub fn save() {
        let state = STATE.with(|state| mem::take(&mut *state.borrow_mut()));

        let cert_counter = CERT_COUNTER.get();
        let cert_tree = CERTIFIED_TREE.with(|tree| {
            tree.borrow()
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect()
        });
        let stable_cert = StorableCert {
            state,
            cert_counter,
            cert: cert_tree,
        };

        if let Err(e) = storage::stable_save((stable_cert,)) {
            ic_cdk::println!("Failed to save to stable storage: {:?}", e);
        } else {
            ic_cdk::println!("Successfully saved to stable storage.");
        }
    }

    pub fn load() {
        match storage::stable_restore() {
            Ok((StorableCert {
                cert_counter,
                cert,
                state,
            },)) => {
                STATE.with(|state0| *state0.borrow_mut() = state);

                let mut cert_tree = RbTree::new();
                for (key, value) in cert {
                    cert_tree.insert(key, value);
                }
                CERT_COUNTER.with(|counter| counter.set(cert_counter));
                CERTIFIED_TREE.with_borrow_mut(|tree| *tree = cert_tree);
                ic_cdk::println!("Successfully restored from stable storage.");
            }
            Err(e) => {
                ic_cdk::println!("Failed to restore from stable storage: {:?}", e);
            }
        }
    }
}

pub mod cert {
    use super::*;

    pub fn get_cret_info(key: String) -> Option<CertificateInfoWrapper> {
        CERTIFIED_STORE.with(|r| r.borrow().get(&key))
    }

    pub fn add_cret_info(key: String, cert_info: CertificateInfo) {
        CERTIFIED_STORE.with(|r| {
            r.borrow_mut()
                .insert(key, CertificateInfoWrapper(cert_info))
        });
    }
}
