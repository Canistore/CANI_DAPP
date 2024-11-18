use candid::{CandidType, Deserialize, Principal};

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct MusicCertificate {
    pub title: String,
    pub artist: String,
    pub owner: Principal,
    pub unique_id: String,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct MusicCertificateResp {
    pub key: String,
    pub music_cert_hex: String,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct MusicCopyright {
    pub count: u128,
    pub certificate: Vec<u8>,
    pub music_cert_hex: String,
    pub witness: Vec<u8>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct CertificateInfo {
    pub key: String,
    pub cert_info: MusicCertificate,
    pub cert_hex: String,
}