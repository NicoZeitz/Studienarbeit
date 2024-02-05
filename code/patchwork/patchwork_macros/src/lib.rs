extern crate proc_macro;
use std::collections::HashSet;

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{bracketed, parenthesized, parse::Parse, parse_macro_input, Token};

mod kw {
    syn::custom_keyword!(patch);
    syn::custom_keyword!(id);
    syn::custom_keyword!(button_cost);
    syn::custom_keyword!(time_cost);
    syn::custom_keyword!(button_income);
    syn::custom_keyword!(tiles);
}

#[proc_macro]
pub fn generate_patches(input: TokenStream) -> TokenStream {
    let Patches {
        patches,
        tiles,
        normalized_tiles,
        transformations,
    } = parse_macro_input!(input as Patches);

    quote! {
        PatchManager {
            patches: [
                #(#patches),*
            ],
            tiles: [
                #(#tiles),*
            ],
            normalized_tiles: [
                #(#normalized_tiles),*
            ],
            transformations: [
                #(#transformations),*
            ]
        }
    }
    .into()
}

struct Patches {
    pub patches: Vec<Patch>,
    pub tiles: Vec<PatchTiling>,
    pub normalized_tiles: Vec<PatchNormalizedTiling>,
    pub transformations: Vec<PatchTransformations>,
}

struct Patch {
    pub id: u8,
    pub button_cost: u8,
    pub time_cost: u8,
    pub button_income: u8,
}

struct PatchTiling {
    pub tiles: Vec<Vec<u8>>,
}

struct PatchNormalizedTiling {
    pub normalized_tiles: [[u8; 5]; 3],
}

struct PatchTransformations {
    pub transformations: Vec<PatchTransformation>,
}

struct PatchTransformation {
    pub row: u8,
    pub column: u8,
    pub transformation: u8,
    pub tiling: u128,
}

impl Parse for Patches {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut patches = Patches {
            patches: vec![],
            tiles: vec![],
            normalized_tiles: vec![],
            transformations: vec![],
        };

        for (patch, tiling, transformation) in input
            .parse_terminated(Self::parse_single_wrapped, Token![,])?
            .into_iter()
        {
            patches.patches.push(patch);
            patches.normalized_tiles.push(normalize_tiling(&tiling.tiles));
            patches.tiles.push(tiling);
            patches.transformations.push(transformation);
        }

        Ok(patches)
    }
}

impl Patches {
    fn parse_single_wrapped(input: syn::parse::ParseStream) -> syn::Result<(Patch, PatchTiling, PatchTransformations)> {
        input.parse::<kw::patch>()?;
        let content;
        parenthesized!(content in input);
        let (patch, tiling, transformation) = Patches::parse_single(&content)?;
        Ok((patch, tiling, transformation))
    }

    fn parse_single(input: syn::parse::ParseStream) -> syn::Result<(Patch, PatchTiling, PatchTransformations)> {
        let id = Patches::parse_property::<kw::id>(input)?;
        let button_cost = Patches::parse_property::<kw::button_cost>(input)?;
        let time_cost = Patches::parse_property::<kw::time_cost>(input)?;
        let button_income = Patches::parse_property::<kw::button_income>(input)?;
        let tiles = Patches::parse_tiles(input)?;
        let transformations = generate_transformations(&tiles);

        let patch = Patch {
            id,
            button_cost,
            time_cost,
            button_income,
        };
        let tiling = PatchTiling { tiles };
        let transformation = PatchTransformations { transformations };

        Ok((patch, tiling, transformation))
    }

    fn parse_property<Keyword: syn::parse::Parse>(input: syn::parse::ParseStream) -> syn::Result<u8> {
        input.parse::<Keyword>()?;
        input.parse::<Token![:]>()?;
        let value: u8 = input.parse::<syn::LitInt>()?.base10_parse()?;
        input.parse::<Token![,]>()?;
        Ok(value)
    }

    fn parse_tiles(input: syn::parse::ParseStream) -> syn::Result<Vec<Vec<u8>>> {
        let content;
        input.parse::<kw::tiles>()?;
        input.parse::<Token![:]>()?;
        bracketed!(content in input);
        let tiles = content.parse_terminated(Patches::parse_single_row, Token![,])?;
        Ok(tiles.into_iter().collect())
    }

    fn parse_single_row(input: syn::parse::ParseStream) -> syn::Result<Vec<u8>> {
        let content;
        bracketed!(content in input);
        let row = content.parse_terminated(Patches::parse_single_tile, Token![,])?;
        Ok(row.into_iter().collect())
    }

    fn parse_single_tile(input: syn::parse::ParseStream) -> syn::Result<u8> {
        let value: u8 = input.parse::<syn::LitInt>()?.base10_parse()?;
        if value != 0 && value != 1 {
            return Err(syn::Error::new(input.span(), "Tile value must be 0 or 1"));
        }
        Ok(value)
    }
}

impl ToTokens for Patch {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Patch {
            id,
            button_cost,
            time_cost,
            button_income,
        } = self;

        tokens.extend(quote! {
            Patch {
                id: #id,
                button_cost: #button_cost,
                time_cost: #time_cost,
                button_income: #button_income,
            }
        });
    }
}

impl ToTokens for PatchTiling {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let PatchTiling { tiles } = self;
        let quoted = tiles.iter().map(|tiling| {
            quote! {
                vec![
                    #(#tiling),*
                ]
            }
        });

        tokens.extend(quote! {
            vec![
                #(#quoted),*
            ]
        });
    }
}

impl ToTokens for PatchNormalizedTiling {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let PatchNormalizedTiling { normalized_tiles } = self;
        let quoted = normalized_tiles.iter().map(|tiling| {
            quote! {
                [
                    #(#tiling),*
                ]
            }
        });

        tokens.extend(quote! {
            [
                #(#quoted),*
            ]
        });
    }
}

impl ToTokens for PatchTransformations {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let PatchTransformations { transformations } = self;
        tokens.extend(quote! {
            vec![
                #(#transformations),*
            ]
        });
    }
}

impl ToTokens for PatchTransformation {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let PatchTransformation {
            row,
            column,
            transformation,
            tiling,
        } = self;
        let tiling_literal = format!("{:#083b}u128", tiling).parse::<proc_macro2::Literal>().unwrap();

        tokens.extend(quote! {
            PatchTransformation {
                row: #row,
                column: #column,
                transformation: #transformation,
                tiles: #tiling_literal,
            }
        });
    }
}

fn generate_transformations(tiles: &Vec<Vec<u8>>) -> Vec<PatchTransformation> {
    let mut saved_transformations = HashSet::<u128>::new();
    let mut transformations: Vec<PatchTransformation> = vec![];

    'transformationLoop: for transformation in 0..=7 {
        let transformed_tiles = get_transformed_tiles(tiles, transformation);
        for row in 0..=9 - transformed_tiles.len() {
            for column in 0..=9 - transformed_tiles[0].len() {
                let mut tiling = 0u128;
                for (i, tile_row) in transformed_tiles.iter().enumerate() {
                    for (j, tile) in tile_row.iter().enumerate() {
                        if *tile == 1 {
                            tiling |= 1u128 << ((row + i) * 9 + (column + j));
                        }
                    }
                }
                if saved_transformations.contains(&tiling) {
                    continue 'transformationLoop;
                }
                saved_transformations.insert(tiling);
                transformations.push(PatchTransformation {
                    row: row as u8,
                    column: column as u8,
                    transformation,
                    tiling,
                });
            }
        }
    }

    transformations
}

struct Constants;

impl Constants {
    pub const ROTATION_0: u8 = 0b000;
    pub const ROTATION_90: u8 = 0b001;
    pub const ROTATION_180: u8 = 0b010;
    pub const ROTATION_270: u8 = 0b011;
    pub const FLIPPED: u8 = 0b100;
    pub const FLIPPED_ROTATION_90: u8 = 0b101;
    pub const FLIPPED_ROTATION_180: u8 = 0b110;
    pub const FLIPPED_ROTATION_270: u8 = 0b111;
}

fn get_transformed_tiles(tiles: &Vec<Vec<u8>>, transformation: u8) -> Vec<Vec<u8>> {
    match transformation {
        Constants::ROTATION_0 => tiles.clone(),
        Constants::ROTATION_90 => {
            let mut new_tiles = vec![vec![0; tiles.len()]; tiles[0].len()];
            for (i, tile_row) in tiles.iter().enumerate() {
                for (j, tile) in tile_row.iter().enumerate() {
                    new_tiles[j][tiles.len() - i - 1] = *tile;
                }
            }
            new_tiles
        }
        Constants::ROTATION_180 => {
            let mut new_tiles = vec![vec![0; tiles[0].len()]; tiles.len()];
            for (i, tile_row) in tiles.iter().enumerate() {
                for (j, tile) in tile_row.iter().enumerate() {
                    new_tiles[tiles.len() - i - 1][tile_row.len() - j - 1] = *tile;
                }
            }
            new_tiles
        }
        Constants::ROTATION_270 => {
            let mut new_tiles = vec![vec![0; tiles.len()]; tiles[0].len()];
            for (i, tile_row) in tiles.iter().enumerate() {
                for (j, tile) in tile_row.iter().enumerate() {
                    new_tiles[tile_row.len() - j - 1][i] = *tile;
                }
            }
            new_tiles
        }
        Constants::FLIPPED => {
            let mut new_tiles = tiles.clone();
            new_tiles.reverse();
            new_tiles
        }
        Constants::FLIPPED_ROTATION_90 => {
            let mut flipped_tiles = tiles.clone();
            flipped_tiles.reverse();

            let mut new_tiles = vec![vec![0; flipped_tiles.len()]; flipped_tiles[0].len()];
            for (i, tile_row) in flipped_tiles.iter().enumerate() {
                for (j, tile) in tile_row.iter().enumerate() {
                    new_tiles[j][flipped_tiles.len() - i - 1] = *tile;
                }
            }
            new_tiles
        }
        Constants::FLIPPED_ROTATION_180 => {
            let mut flipped_tiles = tiles.clone();
            flipped_tiles.reverse();

            let mut new_tiles = vec![vec![0; flipped_tiles[0].len()]; flipped_tiles.len()];
            for (i, tile_row) in flipped_tiles.iter().enumerate() {
                for (j, tile) in tile_row.iter().enumerate() {
                    new_tiles[flipped_tiles.len() - i - 1][tile_row.len() - j - 1] = *tile;
                }
            }
            new_tiles
        }
        Constants::FLIPPED_ROTATION_270 => {
            let mut flipped_tiles = tiles.clone();
            flipped_tiles.reverse();

            let mut new_tiles = vec![vec![0; flipped_tiles.len()]; flipped_tiles[0].len()];
            for (i, tile_row) in flipped_tiles.iter().enumerate() {
                for (j, tile) in tile_row.iter().enumerate() {
                    new_tiles[tile_row.len() - j - 1][i] = *tile;
                }
            }
            new_tiles
        }
        _ => tiles.clone(),
    }
}

fn normalize_tiling(tiles: &Vec<Vec<u8>>) -> PatchNormalizedTiling {
    let mut tiles = tiles;
    let mut normalized_tiles = [[0u8; 5]; 3];
    let mut new_tiles = vec![vec![0; tiles.len()]; tiles[0].len()];

    // if amount rows > 3 we need to rotate the tiling 90°
    if tiles.len() > 3 {
        for (i, tile_row) in tiles.iter().enumerate() {
            for (j, tile) in tile_row.iter().enumerate() {
                new_tiles[j][tiles.len() - i - 1] = *tile;
            }
        }
        tiles = &new_tiles;
    }

    // if amount rows < 3 we need to skip filling the first n rows
    // the same for columns < 5
    let row_offset = 3 - tiles.len();
    let col_offset = 5 - tiles[0].len();

    for (i, tile_row) in tiles.iter().enumerate() {
        for (j, tile) in tile_row.iter().enumerate() {
            if *tile == 1 {
                normalized_tiles[i + row_offset][j + col_offset] = 1;
            }
        }
    }

    PatchNormalizedTiling { normalized_tiles }
}
