        pub struct PatchManager {
            <*\tikzmark{patch-line-2-first}*>pub patches: [Patch; 38],
            pub tiles: [Vec<Vec<u8>>; 38],
            pub transformations: [Vec<PatchTrans<*\tikzmark{patch-line-4-end}*>formation>; 38],
        }

pub struct Patch {              pub struct PatchTransformation {
    pub id: u8,                     pub row: u8,
    pub button_cost: u8,            pub column: u8,
    pub time_cost: u8,              pub transformation: u8,
    pub button_income: u8,          pub tiles: u128,
}                               }