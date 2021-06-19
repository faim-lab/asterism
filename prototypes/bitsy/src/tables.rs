use crate::collision::*;
use asterism::{Logic, QueryTable};
use macroquad::math::IVec2;
use std::fmt::Debug;

impl<TileID: Debug + Copy + Eq + Ord, EntID: Copy>
    QueryTable<(ColIdent, TileMapColData<TileID, EntID>)> for TileMapCollision<TileID, EntID>
{
    fn check_predicate(
        &self,
        predicate: impl Fn(&(ColIdent, TileMapColData<TileID, EntID>)) -> bool,
    ) -> Vec<(ColIdent, TileMapColData<TileID, EntID>)> {
        // overallocates but whatever
        let mut idents = Vec::new();

        // tiles
        for (y, row) in self.map.iter().enumerate() {
            for (x, tile) in row.iter().enumerate() {
                if tile.is_some() {
                    let ident = ColIdent::Position(IVec2::new(x as i32, y as i32));
                    let synthesis = self.get_synthesis(ident);
                    let query_over = (ident, synthesis);
                    if predicate(&query_over) {
                        idents.push(query_over);
                    }
                }
            }
        }

        // entities
        let mut ents = (0..self.positions.len())
            .filter_map(|i| {
                let ident = ColIdent::EntIdx(i);
                let query_over = (ident, self.get_synthesis(ident));
                predicate(&query_over).then(|| query_over)
            })
            .collect();
        idents.append(&mut ents);

        idents
    }
}

impl<TileID: Debug, EntID> QueryTable<Contact> for TileMapCollision<TileID, EntID> {
    fn check_predicate(&self, predicate: impl Fn(&Contact) -> bool) -> Vec<Contact> {
        self.contacts
            .iter()
            .filter_map(|contact| {
                if predicate(contact) {
                    Some(*contact)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    }
}

use crate::types::*;
use crate::Logics;

pub fn test(logics: &mut Logics) {
    // all contacts between player and character
    let player_contacts = query(logics, |contact| -> bool {
        match contact {
            Contact::Ent(i, _) => logics.collision.metadata[*i].id == CollisionEnt::Player,
            Contact::Tile(i, _) => logics.collision.metadata[*i].id == CollisionEnt::Player,
        }
    });

    for _ in player_contacts {
        println!("player touched a thing");
    }
}

fn query(logics: &mut Logics, predicate: impl Fn(&Contact) -> bool) -> Vec<Contact> {
    logics.collision.check_predicate(predicate)
}
