use crate::{
    components::{debug_name, Name, Player},
    init_state,
    player::{get_player_unwrap, is_player, PLAYER_NAME},
};
use specs::{Entities, Entity, WorldExt};

pub fn pluralize_verb<S: ToString>(word: S) -> String {
    word.to_string() + "s"
}

pub fn pluralize_verb_if(pred: bool) -> fn(&str) -> String {
    if pred {
        |word: &str| word.to_string()
    } else {
        |word: &str| pluralize_verb(word)
    }
}

// Write a macro, similar to the Rust format! macro, that accepts the following parameters
// 1. the ecs (of type &World), from which we can get all entities:
//    let entities = ecs.entities();
// 2. subject Entity (from the specs library)
// 3. format string with a subject placeholder <SUBJECT> ? and {} for words to pluralize
// 4. varargs for words to pluralize

// Examples:
// 1. "<SUBJ> {} the apples." -> "You eat the apples." or "The goblin eats the apples."
// 2. "<SUBJ> {} the apples {} and {} the milk." -> "You eat the apples and drink the milk."
//                                           or "The goblin eats the apples and drinks the milk."
//
// Assume we have the following functions available to use:
// is_plural_entity(entities: Entities, subject: Entity) -> bool
// pluralize(entities: Entities, subject: Entity, word: String) -> String
// pub fn get_player_entities_with_pos<P: Join, R: Join>(
//     entities: &Read<EntitiesRes>,
//     players: P,
//     positions: R,
// ) -> Vec<(Entity, Position)>
// where
//     P::Type: IsPlayer,
//     R::Type: Positionable,

// TODO: validate that the format string contains <SUBJ>
macro_rules! entity_action_msg {
    ($entities:expr, $players:expr, $names:expr, $format:literal, $entity:expr $(, $word:tt)+) => {{
        let is_plural = is_player($entities, $players, $entity);
        let debug_name = debug_name();
        let subject = $names.get($entity).unwrap_or(&debug_name);
        let subject_str = if (subject.name == PLAYER_NAME) {
            "You".to_string()
        } else {
            format!("The {}", &subject.name)
        };
        let pluralizer: fn(&str) -> String = pluralize_verb_if(is_plural);
        format!($format, $(pluralizer($word)),+).replace("<SUBJ>", &subject_str)

    }};
}

// TODO: add a variant that hopefully shares the macro code above, but just takes a &World param instead of the 3 params

#[test]
fn pluralize_tests() {
    assert!(pluralize_verb("eat") == "eats");

    let (gs, _) = init_state(true, None);

    let entities = gs.ecs.entities();
    let (monster_entity, _) = {
        let mut names = gs.ecs.write_storage::<Name>();
        // create a monster entity
        let monster_entity = entities.create();
        let name = Name {
            name: "goblin".to_string(),
        };
        // associate the name with the monster entity
        names.insert(monster_entity, name.clone()).unwrap();
        (monster_entity, name)
    };
    let players = gs.ecs.read_storage::<Player>();
    let names = gs.ecs.read_storage::<Name>();
    let player_entity = get_player_unwrap(&gs.ecs, PLAYER_NAME);
    let msg_out1 = entity_action_msg!(
        &entities,
        &players,
        &names,
        "<SUBJ> {} the apple.",
        player_entity,
        "eat"
    );

    assert_eq!(msg_out1, "You eat the apple.");
    let msg_out2 = entity_action_msg!(
        &entities,
        &players,
        &names,
        "<SUBJ> {} the apple.",
        monster_entity,
        "eat"
    );
    assert_eq!(msg_out2, "The goblin eats the apple.");
}
