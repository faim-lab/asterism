use crate::collision::*;
use asterism::{Logic, OutputTable};
use macroquad::math::IVec2;
use std::fmt::Debug;

type QueryIdent<TileID, EntID> = (
    <TileMapCollision<TileID, EntID> as Logic>::Ident,
    <TileMapCollision<TileID, EntID> as Logic>::IdentData,
);

impl<TileID, EntID> OutputTable<QueryIdent<TileID, EntID>> for TileMapCollision<TileID, EntID>
where
    TileID: Debug + Copy + Eq + Ord,
    EntID: Copy,
{
    fn get_table(&self) -> Vec<QueryIdent<TileID, EntID>> {
        let mut idents = Vec::new();

        // tiles
        for (y, row) in self.map.iter().enumerate() {
            for (x, tile) in row.iter().enumerate() {
                if tile.is_some() {
                    let ident = ColIdent::Position(IVec2::new(x as i32, y as i32));
                    idents.push((ident, self.get_synthesis(ident)));
                }
            }
        }

        // entities
        let mut ents = (0..self.positions.len())
            .map(|idx| {
                let ident = ColIdent::EntIdx(idx);
                (ident, self.get_synthesis(ident))
            })
            .collect::<Vec<_>>();
        idents.append(&mut ents);

        idents
    }
}

impl<TileID: Debug, EntID> OutputTable<Contact> for TileMapCollision<TileID, EntID> {
    fn get_table(&self) -> Vec<Contact> {
        self.contacts.to_vec()
    }
}
