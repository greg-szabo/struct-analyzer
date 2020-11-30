use crate::core::consts;
use crate::core::op;

use std::collections::HashMap;
use std::ops::Deref;
use syn::{Attribute, Fields, Item, Visibility};

#[derive(Debug, Copy, Clone)]
pub enum DataType {
    Enum,
    Struct,
    Unknown,
}

impl ToString for DataType {
    fn to_string(&self) -> String {
        match &self {
            DataType::Enum => "ellipse",
            DataType::Struct => "rectangle",
            DataType::Unknown => "rhombus",
        }
        .to_string()
    }
}

#[derive(PartialEq)]
pub enum Color {
    /// Invalid struct or enum
    Red,
    /// No serialization
    White,
    /// Default derive
    Green,
    /// Asymmetric serialization with default derive
    GreenGradient,
    /// Custom implementation
    Yellow,
    /// Asymmetric serialization with custom implementation
    YellowGradient,
    /// RawType
    Blue,
    /// Asymmetric serialization with RawType
    BlueGradient,
}

impl ToString for Color {
    fn to_string(&self) -> String {
        match &self {
            Color::Red => "red",
            Color::White => "white",
            Color::Green => "green",
            Color::GreenGradient => "green_gradient",
            Color::Yellow => "yellow",
            Color::YellowGradient => "yellow_gradient",
            Color::Blue => "blue",
            Color::BlueGradient => "blue_gradient",
        }
        .to_string()
    }
}

#[derive(Debug)]
pub struct Entry {
    public: bool,
    r#type: DataType,
    serialize: bool,
    deserialize: bool,
    serde_from: bool,
    serde_into: bool,

    serializer: bool,
    deserializer: bool,

    serde_custom_field: bool,
    fields: Vec<String>,
}
pub struct Collection(HashMap<String, Entry>);

impl Entry {
    /// constructor
    pub fn new(r#type: DataType) -> Self {
        Self {
            public: false,
            r#type,
            serialize: false,
            deserialize: false,
            serde_from: false,
            serde_into: false,

            serializer: false,
            deserializer: false,

            serde_custom_field: false,
            fields: vec![],
        }
    }

    /// Fill in the basic values based on the input
    pub fn complete_basics(&mut self, vis: &Visibility, attrs: &[Attribute]) {
        if let Visibility::Public(_) = vis {
            self.public = true;
        }
        self.serialize = op::is_ident_with_token_present(attrs, "derive", "Serialize");
        self.deserialize = op::is_ident_with_token_present(attrs, "derive", "Deserialize");
        if op::is_ident_present(attrs, "serde") {
            self.serde_from = op::is_ident_with_token_present(&attrs, "serde", "try_from")
                || op::is_ident_with_token_present(attrs, "serde", "from");
            self.serde_into = op::is_ident_with_token_present(attrs, "serde", "into");
            // Todo: Also save the type.
        }
    }

    /// Fill in the `fields` vector based on the input
    pub fn complete_fields(&mut self, fields: Fields) {
        match fields {
            Fields::Named(n) => {
                if n.named
                    .iter()
                    .any(|f| op::is_ident_present(&f.attrs, "serde"))
                {
                    self.serde_custom_field = true;
                }
                for f in n.named {
                    self.add_to_fields(op::get_idents_from_types(&f.ty));
                }
            }
            Fields::Unnamed(u) => {
                if u.unnamed
                    .iter()
                    .any(|f| op::is_ident_present(&f.attrs, "serde"))
                {
                    self.serde_custom_field = true;
                }
                for f in u.unnamed {
                    self.add_to_fields(op::get_idents_from_types(&f.ty));
                }
            }
            Fields::Unit => {}
        }
    }

    /// Append to the `fields` vector all unique items
    pub fn add_to_fields(&mut self, other: Vec<String>) {
        let mut new_fields = self.fields.clone();
        for o in other {
            if !new_fields.contains(&o) {
                new_fields.push(o);
            }
        }
        self.fields = new_fields;
    }

    /// Get the color (serialization type) of the entry
    pub fn get_color(&self) -> Color {
        let derive = self.serialize || self.deserialize;
        let from_into = self.serde_from || self.serde_into;
        let custom_impl = self.serializer || self.deserializer;
        let gradient = (self.serialize != self.deserialize)
            || (self.serde_from != self.serde_into)
            || (self.serializer != self.deserializer);
        if derive && custom_impl || !derive && from_into {
            Color::Red
        } else if derive {
            if from_into {
                if gradient {
                    Color::BlueGradient
                } else {
                    Color::Blue
                }
            } else if gradient {
                Color::GreenGradient
            } else {
                Color::Green
            }
        } else if custom_impl {
            if gradient {
                Color::YellowGradient
            } else {
                Color::Yellow
            }
        } else {
            Color::White
        }
    }
}

impl Collection {
    /// constructor
    pub fn new() -> Self {
        Self(HashMap::<String, Entry>::new())
    }

    /// Return (get/create/fix) a mutable entry from the collection. If the entry doesn't exist, create it, if the entry type is invalid, fix it.
    pub fn spawn_entry(&mut self, id: &str, new_type: DataType) -> &mut Entry {
        let entry = self
            .0
            .entry(id.to_string())
            .or_insert_with(|| Entry::new(new_type));
        if let DataType::Unknown = entry.r#type {
            entry.r#type = new_type;
        }
        entry
    }

    /// Add Rust tokens into the collection.
    pub fn add_items(&mut self, items: Vec<Item>, id_prefix: &str) {
        for item in items {
            match item {
                Item::Enum(e) => {
                    let id = format!("{}::{}", id_prefix, e.ident);
                    let entry = self.spawn_entry(&id, DataType::Enum);
                    entry.complete_basics(&e.vis, &e.attrs);
                    for variant in e.variants {
                        entry.complete_fields(variant.fields);
                    }
                }
                Item::Struct(e) => {
                    let id = format!("{}::{}", id_prefix, e.ident);
                    let entry = self.spawn_entry(&id, DataType::Struct);
                    entry.complete_basics(&e.vis, &e.attrs);
                    entry.complete_fields(e.fields);
                }
                Item::Impl(i) => {
                    let impl_trait = match i
                        .trait_
                        .as_ref()
                        .map(|(_, path, _)| op::get_idents_from_paths(path).last().cloned())
                        .flatten()
                    {
                        None => continue,
                        Some(t) => t,
                    };
                    let impl_ident =
                        match op::get_idents_from_types(i.self_ty.deref()).last().cloned() {
                            None => continue,
                            Some(n) => n,
                        };
                    let id = format!("{}::{}", id_prefix, impl_ident);
                    match impl_trait.as_str() {
                        "Deserialize" => {
                            self.spawn_entry(&id, DataType::Unknown).deserializer = true
                        }
                        "Serialize" => self.spawn_entry(&id, DataType::Unknown).serializer = true,
                        _ => {}
                    }
                }
                _ => continue,
            }
        }
    }

    fn build_dependencies_for_csv(
        &self,
        collected_item_name: &str,
        collected_item_data: &Entry,
    ) -> (Vec<String>, Vec<String>) {
        let mut solid = Vec::<String>::new();
        let mut dashed = Vec::<String>::new();
        let color = collected_item_data.get_color();

        'fields: for field_being_checked in &collected_item_data.fields {
            for possible_object in self.0.keys() {
                let (collected_item_name_slash, collected_item_name_mid, collected_item_name_last) =
                    path_parts(collected_item_name);
                let (field_being_checked_slash, field_being_checked_mid, field_being_checked_last) =
                    path_parts(field_being_checked);
                let field_being_checked_prefix =
                    joiner(&field_being_checked_slash, &field_being_checked_mid, "/");
                let field_being_checked_object =
                    joiner(&field_being_checked_mid, &field_being_checked_last, "::");
                let collected_item_name_prefix =
                    joiner(&collected_item_name_slash, &collected_item_name_mid, "/");
                let _collected_item_name_object =
                    joiner(&collected_item_name_mid, &collected_item_name_last, "::");

                let is_field_being_checked_simple = field_being_checked_prefix.is_empty();
                let is_field_being_checked_relative = field_being_checked_slash.is_empty();

                // Field matches exactly:
                // item: config::P2PConfig, field: net::Address -> net::Address
                // item: config::RpcConfig, field: net::Address -> net::Address
                // item: config::TendermintConfig, field: net::Address -> net::Address
                // item: config/priv_validator_key::PrivValidatorKey, field: account::Id -> account::Id
                // item: genesis::Genesis, field: validator::Info -> validator::Info
                // item: abci/responses::EndBlock, field: validator::Update -> validator::Update
                if possible_object == field_being_checked {
                    pusher(possible_object, &color, &mut solid, &mut dashed);
                    continue 'fields;
                }

                // Field is in the same file as struct:
                // item: block::Block, field: Header -> block::Header
                // item: config::TxIndexConfig, field: TxIndexer -> config::TxIndexer ???
                // item: validator::Set, field: Info -> validator::Info ???
                // item: abci/responses::Responses, field: DeliverTx -> abci/responses::DeliverTx
                // item: merkle/proof::Proof, field: ProofOp -> merkle/proof::ProofOp
                // item: consensus/params::Params, field: ValidatorParams -> consensus/params::ValidatorParams
                // item: abci/responses::DeliverTx, field: Event -> abci/responses::Event
                // item: config::P2PConfig, field: TransferRate -> config::TransferRate
                // item: abci/tag::Tag, field: Key -> abci/tag::Key
                // item: config::TendermintConfig, field: DbBackend -> config::DbBackend
                // ...
                if is_field_being_checked_simple
                    && possible_object
                        == &joiner(&collected_item_name_prefix, &field_being_checked_last, "::")
                {
                    pusher(possible_object, &color, &mut solid, &mut dashed);
                    continue 'fields;
                }

                // Field is relative and shortened:
                // item: config::P2PConfig, field: Timeout -> timeout::Timeout
                // item: config::ConsensusConfig, field: Timeout -> timeout::Timeout
                // item: validator::Info, field: vote::Power -> vote/power::Power
                // item: consensus/state::State, field: block::Height -> block/height::Height
                // item: abci/responses::EndBlock, field: consensus::Params -> consensus/params::Params
                // item: genesis::Genesis, field: Time -> time::Time
                // ...
                if is_field_being_checked_relative
                    && possible_object
                        == &joiner(
                            &joiner(
                                &field_being_checked_prefix,
                                &op::camelcase_to_snakecase(&field_being_checked_last),
                                "/",
                            ),
                            &field_being_checked_last,
                            "::",
                        )
                {
                    pusher(possible_object, &color, &mut solid, &mut dashed);
                    continue 'fields;
                }

                // Field is in the same module as struct, but in different file and shortened. (sub)
                // item: block::Block, field: Header -> block/header::Header
                // item: channel::Channel, field: Id -> channel/id::Id
                if is_field_being_checked_simple
                    && possible_object
                        == &joiner(
                            &joiner(
                                &collected_item_name_prefix,
                                &op::camelcase_to_snakecase(&field_being_checked_last),
                                "/",
                            ),
                            &field_being_checked_last,
                            "::",
                        )
                {
                    pusher(possible_object, &color, &mut solid, &mut dashed);
                    continue 'fields;
                }

                // Field is in the same module as struct, but in different file and shortened. (super)
                // item: block/commit::Commit, field: Height -> block/height::Height
                if is_field_being_checked_simple
                    && possible_object
                        == &joiner(
                            &joiner(
                                &collected_item_name_slash,
                                &op::camelcase_to_snakecase(&field_being_checked_last),
                                "/",
                            ),
                            &field_being_checked_last,
                            "::",
                        )
                {
                    pusher(possible_object, &color, &mut solid, &mut dashed);
                    continue 'fields;
                }

                // ABCI exception #1
                // item: config::TxIndexConfig, field: tag::Key -> abci/tag::Key
                // item: block::Block, field: transaction::Data -> abci/transaction::Data
                // ...
                if is_field_being_checked_relative
                    && possible_object
                        == &joiner(&"abci".to_string(), &field_being_checked_object, "/")
                {
                    pusher(possible_object, &color, &mut solid, &mut dashed);
                    continue 'fields;
                }

                // ABCI exception #2
                // item: abci/responses::BeginBlock, field: Tag -> abci/tag::Tag
                // item: abci/responses::EndBlock, field: Tag -> abci/tag::Tag
                // item: abci/responses::Event, field: Tag -> abci/tag::Tag
                if is_field_being_checked_simple
                    && possible_object
                        == &joiner(
                            &joiner(
                                &"abci".to_string(),
                                &op::camelcase_to_snakecase(&field_being_checked_last),
                                "/",
                            ),
                            &field_being_checked_last,
                            "::",
                        )
                {
                    pusher(possible_object, &color, &mut solid, &mut dashed);
                    continue 'fields;
                }

                // Channel/s exception
                // item: node/info::Info, field: Channels -> channel::Channels
                if collected_item_name == "node/info::Info"
                    && field_being_checked == "Channels"
                    && possible_object == "channel::Channels"
                {
                    pusher(possible_object, &color, &mut solid, &mut dashed);
                    continue 'fields;
                }

                // PartSetHeader exception
                // item: block/id::Id, field: PartSetHeader -> block/parts::Header
                if collected_item_name == "block/id::Id"
                    && field_being_checked == "PartSetHeader"
                    && possible_object == "block/parts::Header"
                {
                    pusher(possible_object, &color, &mut solid, &mut dashed);
                    continue 'fields;
                }

                // super::Type exception
                // item: vote/canonical_vote::CanonicalVote, field: super::Type -> vote::Type
                if collected_item_name == "vote/canonical_vote::CanonicalVote"
                    && field_being_checked == "super::Type"
                    && possible_object == "vote::Type"
                {
                    pusher(possible_object, &color, &mut solid, &mut dashed);
                    continue 'fields;
                }

                // ChainId exception
                // item: vote/canonical_vote::CanonicalVote, field: ChainId -> chain/id::Id
                if field_being_checked == "ChainId" && possible_object == "chain/id::Id" {
                    pusher(possible_object, &color, &mut solid, &mut dashed);
                    continue 'fields;
                }

                // Height exception
                // item: proposal/canonical_proposal::CanonicalProposal, field: Height -> block/height::Height
                if field_being_checked == "Height" && possible_object == "block/height::Height" {
                    pusher(possible_object, &color, &mut solid, &mut dashed);
                    continue 'fields;
                }

                // Round exception
                // item: proposal::Proposal, field: Round -> block/round::Round
                if field_being_checked == "Round" && possible_object == "block/round::Round" {
                    pusher(possible_object, &color, &mut solid, &mut dashed);
                    continue 'fields;
                }

                // BlockId exception
                // item: proposal/canonical_proposal::CanonicalProposal, field: BlockId -> block/id::Id
                if field_being_checked == "BlockId" && possible_object == "block/id::Id" {
                    pusher(possible_object, &color, &mut solid, &mut dashed);
                    continue 'fields;
                }

                // SignedHeader exception
                // item: evidence::ConflictingHeadersEvidence, field: SignedHeader -> block/signed_header::SignedHeader
                if field_being_checked == "SignedHeader"
                    && possible_object == "block/signed_header::SignedHeader"
                {
                    pusher(possible_object, &color, &mut solid, &mut dashed);
                    continue 'fields;
                }

                //
                // Values to skip (specific cases)
                //

                if collected_item_name == "genesis::Genesis" && field_being_checked == "AppState" {
                    // AppState is a trait with the bound serde_json::Value
                    continue 'fields;
                }

                if (collected_item_name == "private_key::PrivateKey"
                    || collected_item_name == "public_key::PublicKey")
                    && field_being_checked == "Ed25519"
                {
                    // custom serde serialization implemented for external type
                    continue 'fields;
                }

                if collected_item_name == "public_key::PublicKey"
                    && field_being_checked == "Secp256k1"
                {
                    // custom serde serialization implemented for external type
                    continue 'fields;
                }

                if collected_item_name == "timeout::Timeout" && field_being_checked == "Duration" {
                    // custom serde serialization implemented for external type
                    continue 'fields;
                }

                if collected_item_name == "time::Time"
                    && (field_being_checked == "Utc" || field_being_checked == "DateTime")
                {
                    // serde serialization implemented using raw type for external type
                    continue 'fields;
                }

                if (collected_item_name == "proposal/sign_proposal::SignedProposalResponse"
                    || collected_item_name == "vote/sign_vote::SignedVoteResponse"
                    || collected_item_name == "public_key/pub_key_response::PubKeyResponse")
                    && field_being_checked == "RemoteSignerError"
                {
                    // no serialization implemented for external type
                    continue 'fields;
                }

                if collected_item_name == "signature::Signature"
                    && field_being_checked == "Ed25519Signature"
                {
                    // no serialization implemented for external type
                    continue 'fields;
                }

                if collected_item_name == "validator::SimpleValidator"
                    && field_being_checked == "tendermint_proto::crypto::PublicKey"
                {
                    // no serialization implemented for SimpleValidator
                    continue 'fields;
                }
            }
            panic!(
                "could not parse struct or enum: {}, field: {}",
                collected_item_name, field_being_checked
            );
        }
        (solid, dashed)
    }

    /// Parse collection into CSV data.
    pub fn parse_to_csv(&self, only_json: bool, no_header: bool) -> String {
        let mut result = String::new();
        if !no_header {
            result.push_str(consts::HEADER);
            result.push_str("\n");
        }
        let only_public = true;

        for (collected_item_name, collected_item_data) in &self.0 {
            if only_public && !collected_item_data.public {
                continue;
            }

            if only_json && collected_item_data.get_color() == Color::White {
                continue;
            }

            let (solid, dashed) =
                self.build_dependencies_for_csv(collected_item_name, collected_item_data);

            result.push_str(
                format!(
                    "{},{},{},{:?},{:?}\n",
                    collected_item_name,
                    collected_item_data.r#type.to_string(),
                    collected_item_data.get_color().to_string(),
                    solid.join(","),
                    if only_json {
                        "".to_string()
                    } else {
                        dashed.join(",")
                    },
                )
                .as_str(),
            );
        }
        result
    }
}

fn joiner(a: &str, b: &str, sep: &str) -> String {
    if a.is_empty() {
        b.to_string()
    } else if b.is_empty() {
        a.to_string()
    } else {
        [a, b].join(sep)
    }
}

// block/header::Header -> ("block","header","Header")
// some/path/with::complex::Types -> ("some/path","with::complex","Types")
fn path_parts(path: &str) -> (String, String, String) {
    let mut colon2: Vec<_> = path.split("::").collect();
    let last = colon2
        .pop()
        .unwrap_or_else(|| panic!("received invalid path: {}", path));
    let mut slash_vec = Vec::new();
    if !colon2.is_empty() {
        slash_vec = colon2.remove(0).split('/').collect();
        colon2.insert(
            0,
            slash_vec
                .pop()
                .expect("this can't happen: colon2 ran out of elements"),
        );
    }

    (slash_vec.join("/"), colon2.join("::"), last.to_string())
}

fn pusher(possible_object: &str, color: &Color, solid: &mut Vec<String>, dashed: &mut Vec<String>) {
    if color == &Color::Green || color == &Color::GreenGradient {
        solid.push(possible_object.to_string());
    } else {
        dashed.push(possible_object.to_string());
    }
}
