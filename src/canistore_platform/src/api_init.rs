use crate::store;
use canistore_types::platform::{CanisterArgs, ChannelCategory, MusicChannel, MusicType};
use core::time::Duration;

#[ic_cdk::init]
fn init(args: Option<CanisterArgs>) {
    match args {
        Some(CanisterArgs::Init(args)) => {
            store::state::with_mut(|platform| {
                if !args.name.is_empty() {
                    platform.name = args.name;
                }
                platform.owner = args.owner;
                platform.ecdsa_key_name = args.ecdsa_key_name;
                platform.token_expiration = args.token_expiration;
            });
            if args.init_channel == true {
                let mut id: u64 = 1;
                // Initialize Playlist channels
                let playlist_channels = vec![
                    (
                        "Urban Sounds",
                        ChannelCategory::Playlist,
                        MusicType::HipHop,
                        Some(
                            "https://thebots.fun/caniplay/cover_playlist_urban_sounds.png"
                                .to_string(),
                        ),
                    ),
                    (
                        "Electronic",
                        ChannelCategory::Playlist,
                        MusicType::Electronic,
                        Some(
                            "https://thebots.fun/caniplay/cover_playlist_electronic.png"
                                .to_string(),
                        ),
                    ),
                    (
                        "Rock & Roots",
                        ChannelCategory::Playlist,
                        MusicType::Rock,
                        Some(
                            "https://thebots.fun/caniplay/cover_playlist_rock_roots.png"
                                .to_string(),
                        ),
                    ),
                    (
                        "African Giants",
                        ChannelCategory::Playlist,
                        MusicType::Reggae,
                        Some(
                            "https://thebots.fun/caniplay/cover_playlist_african_giants.png"
                                .to_string(),
                        ),
                    ),
                    (
                        "Island Vibes",
                        ChannelCategory::Playlist,
                        MusicType::Reggae,
                        Some(
                            "https://thebots.fun/caniplay/cover_playlist_island_vibes.png"
                                .to_string(),
                        ),
                    ),
                    (
                        "Latin Heat",
                        ChannelCategory::Playlist,
                        MusicType::Pop,
                        Some(
                            "https://thebots.fun/caniplay/cover_playlist_latin_heat.png"
                                .to_string(),
                        ),
                    ),
                    (
                        "Just Beats",
                        ChannelCategory::Playlist,
                        MusicType::Electronic,
                        Some(
                            "https://thebots.fun/caniplay/cover_playlist_just_beats.png"
                                .to_string(),
                        ),
                    ),
                    (
                        "Podcasts",
                        ChannelCategory::Playlist,
                        MusicType::Other,
                        Some(
                            "https://thebots.fun/caniplay/cover_playlist_podcasts.png".to_string(),
                        ),
                    ),
                ];

                for (name, category, music_type, image) in playlist_channels {
                    let new_channel = MusicChannel::new(
                        id,
                        name.to_string(),
                        args.owner,
                        music_type,
                        Some(category),
                        image,
                    );
                    store::channel::add_channel(id, new_channel);
                    id += 1;
                }

                // Initialize Radio channels
                let radio_channels = vec![
                    
                    (
                        "CANI FM",
                        ChannelCategory::Radio,
                        MusicType::Pop,
                        Some("https://thebots.fun/caniplay/cover_radio_cani_fm.png".to_string()),
                    ),
                    (
                        "ICP FM",
                        ChannelCategory::Radio,
                        MusicType::Other,
                        Some("https://thebots.fun/caniplay/cover_radio_icp_fm.png".to_string()),
                    ),
                    (
                        "CaniTalk",
                        ChannelCategory::Radio,
                        MusicType::Other,
                        Some("https://thebots.fun/caniplay/cover_radio_canitalk.png".to_string()),
                    ),
                    (
                        "Crypto with Kamal",
                        ChannelCategory::Radio,
                        MusicType::Other,
                        Some(
                            "https://thebots.fun/caniplay/cover_radio_crypto_with_kamal.jpeg"
                                .to_string(),
                        ),
                    ),
                ];

                for (name, category, music_type, image) in radio_channels {
                    let new_channel = MusicChannel::new(
                        id,
                        name.to_string(),
                        args.owner,
                        music_type,
                        Some(category),
                        image,
                    );
                    store::channel::add_channel(id, new_channel);
                    id += 1;
                }

                store::state::with_mut(|platform| {
                    platform.next_channel_id = id;
                });
            }
        }
        Some(CanisterArgs::Upgrade(_)) => {
            ic_cdk::trap(
                "Cannot initialize the canister with an Upgrade args. Please provide an Init args.",
            );
        }
        None => {
            ic_cdk::trap("No initialization arguments provided");
        }
    }

    ic_cdk_timers::set_timer(Duration::from_secs(0), || {
        ic_cdk::spawn(async {
            if let Err(err) = store::state::init_ecdsa_public_key().await {
                ic_cdk::println!("Error initializing ECDSA public key: {}", err);
            }
        })
    });
}

#[ic_cdk::pre_upgrade]
fn pre_upgrade() {
    store::state::save();
}

#[ic_cdk::post_upgrade]
fn post_upgrade(args: Option<CanisterArgs>) {
    store::state::load();
    match args {
        Some(CanisterArgs::Upgrade(args)) => {
            store::state::with_mut(|platform| {
                if let Some(owner) = args.owner {
                    platform.owner = owner;
                }
                if let Some(token_expiration) = args.token_expiration {
                    platform.token_expiration = token_expiration;
                }
            });
            store::state::save();
        }
        Some(CanisterArgs::Init(_)) => {
            ic_cdk::trap(
                "Cannot upgrade the canister with an Init args. Please provide an Upgrade args.",
            );
        }
        _ => {}
    }
}
