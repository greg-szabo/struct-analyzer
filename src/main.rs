use quote::__private::TokenTree;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::ops::Deref;
use std::path::PathBuf;
use syn::{Attribute, Item, ItemImpl, Path, Type, Visibility};
use walkdir::WalkDir;

#[derive(Debug, Copy, Clone)]
enum DataType {
    Enum,
    Struct,
    Unknown,
}

#[derive(Debug)]
struct Data {
    public: bool,
    r#type: DataType,
    //fields: Vec<Data>,
    serialize: bool,
    deserialize: bool,
    serializer: bool,
    deserializer: bool,
    serde_from: bool,
    serde_into: bool,
}
struct Collection(HashMap<String, Data>);

impl Data {
    pub fn new(r#type: DataType) -> Self {
        Self {
            public: false,
            r#type,
            serialize: false,
            deserialize: false,
            serializer: false,
            deserializer: false,
            serde_from: false,
            serde_into: false,
        }
    }
}

impl Collection {
    pub fn new() -> Self {
        Self(HashMap::<String, Data>::new())
    }

    pub fn into_hashmap(self) -> HashMap<String, Data> {
        self.0
    }

    /// spawn_entry returns an entry. If the entry didn't exist, it creates it, if the entry type was invalid, it fixes it.
    pub fn spawn_entry(&mut self, id: &str, new_type: DataType) -> &mut Data {
        let entry = self
            .0
            .entry(id.to_string())
            .or_insert_with(|| Data::new(new_type));
        if let DataType::Unknown = entry.r#type {
            entry.r#type = new_type;
        }
        entry
    }
}

fn find_rust_files(rust_path: &str) -> Vec<PathBuf> {
    WalkDir::new(rust_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_file()
                && e.path().extension().is_some()
                && e.path().extension().unwrap() == "rs"
        })
        .map(|e| e.into_path())
        .collect()
}

fn parse_file(path: PathBuf) -> syn::File {
    let mut file = File::open(&path).expect("Unable to open file");
    let mut src = String::new();
    file.read_to_string(&mut src).expect("Unable to read file");
    syn::parse_file(&src).expect("Unable to parse file")
}

fn get_ident(path: &Path) -> Option<String> {
    path.segments
        .iter()
        .last()
        .map(|path_segment| path_segment.ident.to_string())
}

fn is_ident_present(attributes: &[Attribute], ident: &str) -> bool {
    attributes
        .iter()
        .any(|attribute| match get_ident(&attribute.path) {
            Some(i) => i == ident,
            None => false,
        })
}

fn is_ident_with_token_present(attributes: &[Attribute], ident: &str, token: &str) -> bool {
    attributes.iter().any(|attribute| {
        if let Some(i) = get_ident(&attribute.path) {
            if i == ident {
                attribute.tokens.clone().into_iter().any(|tt| match tt {
                    TokenTree::Group(group) => group.stream().into_iter().any(|tt| match tt {
                        TokenTree::Ident(ident) => ident == token,
                        _ => false,
                    }),
                    _ => false,
                })
            } else {
                false
            }
        } else {
            false
        }
    })
}

fn get_impl_trait(item_impl: &ItemImpl) -> Option<String> {
    match &item_impl.trait_ {
        None => None,
        Some(t) => get_ident(&t.1),
    }
}

fn get_impl_ident(item_impl: &ItemImpl) -> Option<String> {
    match item_impl.self_ty.deref() {
        Type::Path(tp) => get_ident(&tp.path),
        _ => None,
    }
}

fn main() {
    // Todo: input parameters
    let rust_path = "/Users/greg/git/informal/tendermint-rs/tendermint/src/";
    let no_header = false;
    let relative_path = false;

    let files = find_rust_files(&rust_path);
    let mut collection: Collection = Collection::new();
    for file in files {
        let syntax = parse_file(file.clone());
        let id_prefix = if relative_path {
            file.file_name().unwrap().to_str()
        } else {
            file.strip_prefix(&rust_path).unwrap().to_str()
        }
        .unwrap()
        .strip_suffix(".rs")
        .unwrap();
        for item in syntax.items {
            match item {
                Item::Enum(e) => {
                    let id = format!("{}::{}", id_prefix, e.ident);
                    let entry = collection.spawn_entry(&id, DataType::Enum);
                    if let Visibility::Public(_) = e.vis {
                        entry.public = true;
                    }
                    entry.serialize = is_ident_with_token_present(&e.attrs, "derive", "Serialize");
                    entry.deserialize =
                        is_ident_with_token_present(&e.attrs, "derive", "Deserialize");
                    if is_ident_present(&e.attrs, "serde") {
                        entry.serde_from =
                            is_ident_with_token_present(&e.attrs, "serde", "try_from")
                                || is_ident_with_token_present(&e.attrs, "serde", "from");
                        entry.serde_into = is_ident_with_token_present(&e.attrs, "serde", "into");
                    }
                }
                Item::Struct(e) => {
                    let id = format!("{}::{}", id_prefix, e.ident);
                    let entry = collection.spawn_entry(&id, DataType::Struct);
                    if let Visibility::Public(_) = e.vis {
                        entry.public = true;
                    }
                    entry.serialize = is_ident_with_token_present(&e.attrs, "derive", "Serialize");
                    entry.deserialize =
                        is_ident_with_token_present(&e.attrs, "derive", "Deserialize");
                    if is_ident_present(&e.attrs, "serde") {
                        entry.serde_from =
                            is_ident_with_token_present(&e.attrs, "serde", "try_from")
                                || is_ident_with_token_present(&e.attrs, "serde", "from");
                        entry.serde_into = is_ident_with_token_present(&e.attrs, "serde", "into");
                    }
                }
                Item::Impl(i) => {
                    let impl_trait = match get_impl_trait(&i) {
                        None => continue,
                        Some(t) => t,
                    };
                    let impl_ident = match get_impl_ident(&i) {
                        None => continue,
                        Some(n) => n,
                    };
                    let id = format!("{}::{}", id_prefix, impl_ident);
                    match impl_trait.as_str() {
                        "Deserialize" => {
                            collection.spawn_entry(&id, DataType::Unknown).deserializer = true
                        }
                        "Serialize" => {
                            collection.spawn_entry(&id, DataType::Unknown).serializer = true
                        }
                        _ => {}
                    }
                }
                _ => continue,
            }
        }
    }
    let collection = collection
        .into_hashmap()
        .into_iter()
        .filter(|c| c.1.public)
        .collect::<HashMap<String, Data>>();

    if !no_header {
        println!(
            r#"## Tendermint public JSON-serializable structures - draw.io CSV export
# label: %name%
# stylename: color
# styles: {{ \
#            "red": "shape=%shape%;rounded=1;fillColor=#f8cecc;strokeColor=#b85450;strokeWidth=2",\
#            "green": "shape=%shape%;rounded=1;fillColor=#d5e8d4;strokeColor=#82b366;strokeWidth=2",\
#            "blue": "shape=%shape%;rounded=1;fillColor=#dae8fc;strokeColor=#6c8ebf;strokeWidth=2",\
#            "yellow": "shape=%shape%;rounded=1;fillColor=#fff2cc;strokeColor=#d6b656;strokeWidth=2",\
#            "white": "shape=%shape%;rounded=1;fillColor=#ffffff;strokeColor=#000000;strokeWidth=2",\
#            "green_gradient": "shape=%shape%;rounded=1;fillColor=#d5e8d4;strokeColor=#82b366;strokeWidth=2;gradientColor=#ffffff",\
#            "blue_gradient": "shape=%shape%;rounded=1;fillColor=#dae8fc;strokeColor=#6c8ebf;strokeWidth=2;gradientColor=#ffffff",\
#            "yellow_gradient": "shape=%shape%;rounded=1;fillColor=#fff2cc;strokeColor=#d6b656;strokeWidth=2;gradientColor=#ffffff",\
#            "legend": "shape=%shape%;rounded=1;shadow=1;fontSize=16;align=left;whiteSpace=wrap;html=1;fillColor=#d0cee2;strokeWidth=2;strokeColor=#56517e;"\
# }}
# connect: {{"from":"refs", "to":"name", "invert":false, "style":"curved=1;endArrow=blockThin;endFill=1;"}}
# namespace: tendermint-
# width: auto
# height: auto
# padding: 15
# ignore: outline,refs
# nodespacing: 40
# levelspacing: 100
# edgespacing: 40
# layout: auto
name,shape,color,note,refs
"<b>LEGEND<br><br><b style=\"color:#d5e8d4;\">Green:</b> #[derive(Deserialize, Serialize)]<br><b style=\"color:#dae8fc;\">Blue:</b> #[serde(try_from = \"\", into = \"\")]<br><b style=\"color:#fff2cc;\">Yellow:</b> impl Deserialize/Serialize for my_struct {{}}<br><b style=\"color:#ffffff;\">White:</b> No serialization<br><br>Gradient color: asymmetric serialization<br>Red: invalid combination of features<br>Rounded rectangle: struct<br>Ellipse: enum</b>",rectangle,legend,"#
        );
    }
    for c in collection {
        // Green: #[derive(Serialize, Deserialize)]
        let derive = c.1.serialize || c.1.deserialize;
        // Blue: #[serde(from = "", into = "")
        let from_into = c.1.serde_from || c.1.serde_into;
        // Yellow: impl Serialize/Deserialize for item {}
        let custom_impl = c.1.serializer || c.1.deserializer;
        // Gradient: asymmetric serialization
        let gradient = (c.1.serialize != c.1.deserialize)
            || (c.1.serde_from != c.1.serde_into)
            || (c.1.serializer != c.1.deserializer);
        // Red: Invalid combination
        let invalid = (derive && custom_impl) || (!derive && from_into);
        let color = if invalid {
            "red"
        } else if derive {
            if gradient {
                "green_gradient"
            } else {
                "green"
            }
        } else if from_into {
            if gradient {
                "blue_gradient"
            } else {
                "blue"
            }
        } else if custom_impl {
            if gradient {
                "yellow_gradient"
            } else {
                "yellow"
            }
        } else {
            "white"
        };

        let shape = match c.1.r#type {
            DataType::Struct => "rectangle",
            DataType::Enum => "ellipse",
            DataType::Unknown => "rhombus",
        };
        println!("{},{},{},node/id::Id", c.0, shape, color);
    }
}
