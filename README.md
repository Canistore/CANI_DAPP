# `Cani DAO`

## Architecture Overview

Cani DAO is a decentralized platform based on the Internet Computer (IC) network, designed for independent musicians to manage music storage and licensing through a DAO organization. The platform consists of several key components:

1. canistore_dao: This module is responsible for managing the DAO organization, including the creation and governance of user music spaces. It facilitates the overall coordination of the Canistore ecosystem, ensuring decentralized decision-making and governance.

2. canistore_space: This component represents the personal music space of an artist. It is where individual musicians can manage their music collections, albums, songs, licensing, and charging services. Each artist’s personal space serves as their dedicated environment for organizing and distributing their creative works.

3. canistore_oss_bucket: This is the cloud storage service for music files, providing a scalable and secure way to store the actual audio content. It supports streaming services, allowing the stored music files to be accessed and distributed efficiently. There is a one-to-many relationship between canistore_space and canistore_oss_bucket, where each artist’s space can link to multiple storage buckets.

4. canistore_user: This is the user management center of the platform. It handles authentication and login services for both musicians and regular users. The platform supports multiple login methods, ensuring flexibility for all types of users to access their accounts.

5. canistore_indexer: This component functions as the platform’s indexing service. It indexes information related to musicians, music resources, and users, enabling efficient matching and querying. The indexer plays a crucial role in powering the search functionality for the music portal, helping users to discover musicians and their works through a well-organized resource index.

```
canistore_dao(DAO)
   |
   |-- Manages & Creates --> canistore_space
                              |
                              |-- One-to-Many --> canistore_oss_bucket (Cloud Storage & Streaming)
                              |
                              |-- Authenticated by --> canistore_user (User & Musician Management)
                              |
                              |-- Indexed by --> canistore_indexer (Resource Indexing)
                                                   |
                                                   |-- Provides Search & Matching --> Music Portal
```

## Resource Authorization Architecture
---------------------------------------

```
+----------------------+              +--------------------+
|                      |              |                    |
|   Client (User/App)  +--------------> Authorization      |
|                      |  Request     | Canister (Verifier)|
+----------------------+   Token      |                    |
                                      +---------+----------+
                                                |
                                                | ICST Token Validation
                                                |
                                      +---------v----------+
                                      |                    |
                                      |  IC ECDSA Signature|
                                      |  Service (Verifier)|
                                      |                    |
                                      +---------+----------+
                                                |
                                                | Signature Validation
                                                |
                                      +---------v----------+
                                      |                    |
                                      |  Resource Access   |
                                      |  Resources / Music |
                                      |                    |
                                      +--------------------+


Flow:
1. The Client requests access to a resource, sending a ICST token.
2. The OSS Authorization Canister verifies the ICST token and forwards the request to the IC ECDSA Signature Service.
3. The IC ECDSA Service validates the signature.
4. If valid, access to the requested resource (Resources / Music) is granted.
```

### IC's ECDSA signature service of Advantages

1. Decentralized Key Management: Enhanced security without centralized points of failure.
2. Seamless Smart Contract Integration: Direct ECDSA support within IC canisters.
3. Cross-Chain Interoperability: Ability to interact with other blockchains like Bitcoin and Ethereum.
4. Cost Efficiency: No need for external services or infrastructure, and free of gas fees.
5. Scalability: High throughput and optimized for large volumes of transactions.
6. Enhanced Security: Leveraging threshold cryptography for superior key protection.
7. Transparency and Auditing: Verifiable and auditable signatures on-chain.

## Music Certificate System Technology and Advantages

### Isntroduce

1. The **music copyright certificate** consists of **music creation data** and the **CRC information** of the music file. This combination ensures both the authenticity of the creative work and the integrity of the music file.

2. The copyright information is hashed using a **cryptographic algorithm** to generate a **copyright hash**.

3. The **copyright hash** is then inserted into a **Merkle tree**. This structure ensures efficient and secure verification of the data.

4. The **root of the Merkle tree** is written into the **Internet Computer’s Certified Data**, making it tamper-proof and ensuring decentralized verification.

5. The **copyright hash** and its corresponding **metadata** are stored in a **DAO canister**, allowing users to query and verify ownership and copyright information easily.

6. Users can verify their copyright information in two ways:
   - By checking the **Merkle tree root** to ensure that the copyright hash is authenticated and valid.
   - By querying the **copyright metadata** stored in the DAO canister to confirm the integrity of the copyright data.

### Architecture

```
+-------------------------------+
|  User Interface & API         |
|  (Request Handler & Logic)    |
+-----------+-------------------+
            |
            |
+-----------v-------------------+
|   Copyright Manager           |
|   (RbTree Storage &           |
|    Manipulation)              |
+-----------+-------------------+
            |
            |
+-----------v-------------------+
|   Music Certificate           |
|   (Generated from user        |
|    input and file hash)       |
+-------------------------------+
            |
            |
+-----------v-------------------+
|   IC Network                  |
|   (Certified Data Storage)    |
|   (Root Hash Storage          |
|    via set_certified_data)    |
+-------------------------------+
```

### Technology

1. **Hashing**: Utilizes cryptographic hashing to generate unique identifiers for music files, ensuring authenticity.

2. **CRC Check**: Adds a cyclic redundancy check (CRC) to verify the integrity of the music files. The CRC value, along with the file’s metadata, ensures that no data corruption occurs during storage or transmission.

3. **Red-Black Tree**: A balanced tree structure that facilitates efficient insertion, deletion, and querying of copyright data.

4. **Certified Data**: Employs the Internet Computer's certification service to store the root hash, making it tamper-proof.

5. **Music Copyright Certificate**: The certificate consists of both the music creation information and the CRC data of the music files. This is hashed using a cryptographic algorithm to generate a copyright hash, which is then written into a Merkle tree. The root of the Merkle tree is stored in IC’s certified data to ensure security.

6. **DAO Canister**: Stores both the music copyright hash and metadata in a decentralized autonomous organization (DAO) canister, enabling users to query and verify ownership and rights. Users can verify the authenticity of the certificate by checking the Merkle tree root or querying the metadata.

7. **API Interaction**: Seamlessly connects user input with backend processes to generate and store certificates.

### Advantages

1. **Data Integrity**: Changes in the copyright data are reflected in the root hash, and with the additional CRC check for music files, unauthorized alterations or corruption are easily detectable.

2. **Efficiency**: The Red-Black Tree allows for quick access and management of a potentially large dataset of music copyrights.

3. **Decentralization**: Reduces reliance on centralized databases, enhancing security and reliability.

4. **Transparency**: Users can verify their copyright information using the data certificate query. The CRC value ensures the music file’s integrity, while the Merkle tree root guarantees that copyright data is certified.

5. **Scalability**: Capable of efficiently managing growing amounts of copyright information and music files without performance degradation.

6. **Security**: Protects copyright holders’ rights through secure storage mechanisms, ensuring that both the copyright data and the music file’s integrity are safeguarded.

7. **Verification**: Users can verify the authenticity of the copyright hash by comparing it against the Merkle tree root or by querying the music metadata from the DAO canister, ensuring that their copyright certificate remains valid and authenticated.

## Decentralized Messaging

### Isntroduce

The Decentralized Messaging System is a decentralized message management solution designed to provide unified message storage and efficient retrieval services for distributed user applications. Each user app canister sends messages through an embedded synchronized message queue to the central message index (Message Index Center Canister), which dynamically indexes and categorizes received messages. The system divides messages into "Recent Messages" and "Historical Messages" categories, where only the latest 6000 messages are kept in the recent messages storage, and messages exceeding this limit are moved to historical storage. This design optimizes retrieval efficiency while ensuring scalable data management.

### Architecture

```
                    +---------------------------------------------+
                    |         Decentralized Messaging             |
                    |               System                        |
                    +---------------------------------------------+
                           |                         |
                           |                         |
                           v                         v
       +---------------------------------+  +---------------------------------+
       |       User App Canister (A)     |  |       User App Canister (B)     |
       +---------------------------------+  +---------------------------------+
       |   +-------------------------+   |  |   +-------------------------+   |
       |   | Synchronized Message    |   |  |   | Synchronized Message    |   |
       |   | Queue                   |   |  |   | Queue                   |   |
       |   +-------------------------+   |  |   +-------------------------+   |
       +---------------------------------+  +---------------------------------+
                           |                          |
                           |                          |
                           v                          v
                   +------------------------------------------------+
                   |          Message Index Center Canister         |
                   +------------------------------------------------+
                   |            +-------------------------+         |
                   |            | Synchronized Message    |         |
                   |            | Queue                   |         |
                   |            +-------------------------+         |
                   +------------------------------------------------+
                           |                           |
                           |                           |
                           v                           v
             +-----------------------+           +-----------------------+
             |  Recent Messages      |           |  Historical Messages  |
             |    Storage (<6000)    |           |    Archive Storage    |
             +-----------------------+           +-----------------------+
```

### Technology

1. **User App Canister**
   - **Overview**: Each user has their own application canister, responsible for generating and sending messages. Inside each `User App Canister` is an embedded **Synchronized Message Queue** module, which securely transfers messages to the Message Index Center.
   - **Modules**:
     - **Synchronized Message Queue**: Embedded within each user app canister, this queue manages the buffering and synchronization of messages. It pushes generated messages to the Message Index Center Canister at regular intervals for unified management and storage.

2. **Message Index Center Canister**
   - **Overview**: This is the central message indexing canister, responsible for managing and storing messages from all user applications. It provides dynamic, aggregated message retrieval services and supports message categorization and historical lookups as needed. The index canister also includes an embedded **Synchronized Message Queue** module to receive messages from the user app canisters.
   - **Modules**:
     - **Synchronized Message Queue**: Embedded in the Message Index Center, this queue receives messages from multiple user app canisters and directs them to the appropriate storage area.
     - **Recent Messages Storage**: Stores currently active messages with a capacity limit of 6000 entries. Only the latest 6000 messages are retained; older messages beyond this threshold are moved to the historical storage area.
     - **Historical Messages Storage**: Archives messages that exceed the 6000-message limit. This module supports historical message retrieval and allows users to query past messages when needed.

### System Features

- **Decentralized Architecture**: The system supports multiple user app canisters and centralizes message management and retrieval through the Message Index Center Canister.
- **Tiered Message Storage**: Dual-layer storage (Recent and Historical) ensures quick message retrieval and allows for excellent storage scalability.
- **Efficient Message Retrieval**: With the combined indexing capabilities of the Message Index Center and synchronized queue modules, the system enables dynamic, aggregated message retrieval, supporting a variety of query needs for users.

## IC-Based OSS Cloud Storage and Music Streaming

The **IC-based OSS Cloud Storage and Music Streaming System** is designed for secure, decentralized storage of large media files, specifically for music and audio. Leveraging the Internet Computer (IC) infrastructure, this solution utilizes decentralized Object Storage Service (OSS) canisters to store and retrieve audio data, while implementing **HttpStreamingResponse** for efficient, on-demand music streaming to users.

### System Components

1. **OSS Cloud Storage Canisters**
   - **Decentralized Storage**: The OSS canisters provide decentralized storage for audio files, ensuring data security, scalability, and cost-efficiency. Each audio file is broken down into chunks and stored across distributed canisters on the IC, making the system resistant to single points of failure and highly available.
   - **Metadata Management**: Alongside the audio data, each file’s metadata (e.g., title, artist, duration, format) is stored, enabling organized retrieval and integration with streaming services.
   - **File Access Control**: Each file can be restricted based on access permissions, providing privacy controls suitable for subscription-based music applications.

2. **HttpStreamingResponse for Music Streaming**
   - **On-Demand Streaming**: Using the **HttpStreamingResponse** feature of the Internet Computer, the system streams audio data in real-time, retrieving file chunks sequentially from the OSS canisters. This minimizes buffer time and enables immediate playback, improving the user experience.
   - **Chunked Data Delivery**: HttpStreamingResponse divides the audio file into small chunks, which are transmitted sequentially to the user’s client. This chunking method allows for smooth, uninterrupted playback, even on varying network speeds.
   - **Adaptive Bitrate Streaming**: For optimization, the system can adapt the streaming bitrate based on network conditions, delivering a high-quality audio experience without overloading the network.
   - **Playback Control**: The streaming system also provides support for advanced playback controls, such as seeking, pausing, and resuming, allowing users to interact with the music as they wish.

### Key Benefits

- **Decentralized and Secure Storage**: OSS canisters on the IC offer a decentralized approach, ensuring that audio files are securely stored and distributed, reducing reliance on traditional, centralized cloud storage.
- **Optimized Streaming**: HttpStreamingResponse provides efficient, real-time streaming with minimal latency, ensuring that users enjoy smooth and uninterrupted music playback.
- **Scalable and Cost-Effective**: IC’s decentralized model scales with demand and provides cost savings, making it a sustainable choice for high-volume music streaming services.
- **User-Friendly Playback Controls**: With adaptive bitrate and interactive playback controls, the system meets modern audio streaming standards, enhancing the user experience across various network conditions and devices.

This approach showcases how IC can revolutionize cloud storage and media streaming, offering a robust alternative to centralized storage providers and traditional streaming architectures.

## System Deploy

### canistore_dao

```bash

dfx deploy canistore_dao --argument '(opt record { name = "Canistore DAO"; owner = principal "xr4tj-xppkb-gimlp-62puo-pyheq-ksj3q-5p3fn-x6bwp-pellg-isj7a-6qe"; user_canister_id = principal "bd3sg-teaaa-aaaaa-qaaba-cai" })'

```

### canistore_platform

```bash

dfx deploy canistore_platform --argument "(opt variant { Init = record { name = \"Canistore Platform\"; owner = principal \"xr4tj-xppkb-gimlp-62puo-pyheq-ksj3q-5p3fn-x6bwp-pellg-isj7a-6qe\"; ecdsa_key_name = \"key_1\"; token_expiration = 86400; init_channel = true } })" --network ic

dfx deploy canistore_platform --argument "(opt variant { Upgrade = record { owner = opt principal \"xr4tj-xppkb-gimlp-62puo-pyheq-ksj3q-5p3fn-x6bwp-pellg-isj7a-6qe\"; token_expiration = opt 86400 } })"\


```

### canistore_user

```bash

dfx deploy canistore_user --argument '(opt record { name = "Canistore User Center"; owner = principal "xr4tj-xppkb-gimlp-62puo-pyheq-ksj3q-5p3fn-x6bwp-pellg-isj7a-6qe"; dao_canister_id = principal "vght4-jyaaa-aaaag-aceyq-cai"; indexer_canister_id = null })'

```


## Backend canister via Candid interface

### PROD

canistore_dao: https://a4gq6-oaaaa-aaaab-qaa4q-cai.raw.icp0.io/?id=tlcef-5iaaa-aaaas-akjmq-cai

canistore_indexer: https://a4gq6-oaaaa-aaaab-qaa4q-cai.raw.icp0.io/?id=tqhya-hqaaa-aaaas-akjoa-cai

canistore_platform: https://a4gq6-oaaaa-aaaab-qaa4q-cai.raw.icp0.io/?id=tfajn-gyaaa-aaaas-akjnq-cai

canistore_user: https://a4gq6-oaaaa-aaaab-qaa4q-cai.raw.icp0.io/?id=tcbpz-laaaa-aaaas-akjna-cai

```

## Other

Welcome to your new `canistore` project and to the Internet Computer development community. By default, creating a new project adds this README and some template files to your project directory. You can edit these template files to customize your project and to include your own code to speed up the development cycle.

To get started, you might want to explore the project directory structure and the default configuration file. Working with this project in your development environment will not affect any production deployment or identity tokens.

To learn more before you start working with `canistore`, see the following documentation available online:

- [Quick Start](https://internetcomputer.org/docs/current/developer-docs/setup/deploy-locally)
- [SDK Developer Tools](https://internetcomputer.org/docs/current/developer-docs/setup/install)
- [Rust Canister Development Guide](https://internetcomputer.org/docs/current/developer-docs/backend/rust/)
- [ic-cdk](https://docs.rs/ic-cdk)
- [ic-cdk-macros](https://docs.rs/ic-cdk-macros)
- [Candid Introduction](https://internetcomputer.org/docs/current/developer-docs/backend/candid/)

If you want to start working on your project right away, you might want to try the following commands:

```bash
cd canistore/
dfx help
dfx canister --help
```

## Running the project locally

If you want to test your project locally, you can use the following commands:

```bash
# Starts the replica, running in the background
dfx start --background

# Deploys your canisters to the replica and generates your candid interface
dfx deploy
```

Once the job completes, your application will be available at `http://localhost:4943?canisterId={asset_canister_id}`.

If you have made changes to your backend canister, you can generate a new candid interface with

```bash
npm run generate
```

at any time. This is recommended before starting the frontend development server, and will be run automatically any time you run `dfx deploy`.

If you are making frontend changes, you can start a development server with

```bash
npm start
```

Which will start a server at `http://localhost:8080`, proxying API requests to the replica at port 4943.

### Note on frontend environment variables

If you are hosting frontend code somewhere without using DFX, you may need to make one of the following adjustments to ensure your project does not fetch the root key in production:

- set`DFX_NETWORK` to `ic` if you are using Webpack
- use your own preferred method to replace `process.env.DFX_NETWORK` in the autogenerated declarations
  - Setting `canisters -> {asset_canister_id} -> declarations -> env_override to a string` in `dfx.json` will replace `process.env.DFX_NETWORK` with the string in the autogenerated declarations
- Write your own `createActor` constructor
