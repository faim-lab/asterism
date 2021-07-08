use crate::collision::*;
use asterism::QueryTable;
use macroquad::math::IVec2;
use std::fmt::Debug;

impl<TileID, EntID> QueryTable<ColIdent> for TileMapCollision<TileID, EntID>
where
    TileID: Debug + Copy + Eq + Ord,
    EntID: Copy,
{
    fn get_table(&self) -> Vec<ColIdent> {
        let mut idents = Vec::new();

        // tiles
        for (y, row) in self.map.iter().enumerate() {
            for (x, tile) in row.iter().enumerate() {
                if tile.is_some() {
                    idents.push(ColIdent::Position(IVec2::new(x as i32, y as i32)));
                }
            }
        }

        // entities
        let mut ents = (0..self.positions.len())
            .map(ColIdent::EntIdx)
            .collect::<Vec<_>>();
        idents.append(&mut ents);

        idents
    }
}

impl<TileID: Debug, EntID> QueryTable<Contact> for TileMapCollision<TileID, EntID> {
    fn get_table(&self) -> Vec<Contact> {
        self.contacts.to_vec()
    }
}
