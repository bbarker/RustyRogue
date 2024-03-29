use specs::{Entities, ReadStorage};

use crate::components::{Name, Player};

pub struct EcsActionMsgData<'a> {
    pub entities: &'a Entities<'a>,
    pub players: &'a ReadStorage<'a, Player>,
    pub names: &'a ReadStorage<'a, Name>,
}
impl<'a> EcsActionMsgData<'a> {
    pub fn new(
        entities: &'a Entities<'a>,
        players: &'a ReadStorage<'a, Player>,
        names: &'a ReadStorage<'a, Name>,
    ) -> Self {
        Self {
            entities,
            players,
            names,
        }
    }
}

/// A macro similar to the Rust `format!` macro with additional functionality for pluralization and subject replacement.
///
/// Parameters:
/// 1. `entities`: ecs entities
/// 2. `players`: ecs player components
/// 3. `names`: ecs name components
/// 4. `format_string`: a string with a subject placeholder `<SUBJ>` and `{}` for words to pluralize
/// 5. `entity`: the entity that is the subject of the message
/// 6. `word`+: words to potentially pluralize
///
/// # Examples
/// ```
/// // Example 1:
/// // "<SUBJ> {} the apples." -> "You eat the apples." or "The goblin eats the apples."
///
/// // Example 2:
/// // "<SUBJ> {} the apples {} and {} the milk." -> "You eat the apples and drink the milk."
/// // or "The goblin eats the apples and drinks the milk."
/// ```
///
/// # TODO
/// Validate that the `format_string` contains `<SUBJ>`.
#[macro_export]
macro_rules! entity_action_msg_no_ecs {
    ($ecs_data:expr, $format:literal, $entity:expr $(, $word:tt)+) => {{
        let (entities, players, names) = ($ecs_data.entities, $ecs_data.players, $ecs_data.names);
        let is_plural = $crate::player::is_player(entities, players, $entity);
        let debug_name = $crate::components::debug_name();
        let subject = names.get($entity).unwrap_or(&debug_name);
        let subject_str = if (subject.name == $crate::player::PLAYER_NAME) {
            "You".to_string()
        } else {
            format!("The {}", &subject.name)
        };
        let pluralizer: fn(&str) -> String = $crate::util::pluralize_verb_if(is_plural);
        // FIXME: we don't do a compile-time check on <SUBJ> currently;
        // see end of chat here: https://chat.openai.com/c/e771feb8-8c4e-4dd2-8fc0-5a002e204225
        format!($format, $(pluralizer($word)),+).replace("<SUBJ>", &subject_str)

    }};
}

/// A macro similar to the Rust `format!` macro with additional functionality for pluralization and subject replacement.
/// This macro is used when the `ecs` (World) is available. See `entity_action_msg_no_ecs!` for more details.
#[macro_export]
macro_rules! entity_action_msg {
    ($ecs:expr, $format:literal, $entity:expr $(, $word:tt)+) => {{
        let entities = $ecs.entities();
        let players = $ecs.read_storage::<Player>();
        let names = $ecs.read_storage::<Name>();
        let ecs_data = $crate::util_ecs::EcsActionMsgData::new(&entities, &players, &names);
        $crate::entity_action_msg_no_ecs!(ecs_data, $format, $entity $(, $word)+)
    }};
}

#[cfg(test)]
mod tests {
    use crate::{
        components::{Name, Player},
        init_state,
        player::{get_player_unwrap, PLAYER_NAME},
        util::pluralize_verb,
        util_ecs::EcsActionMsgData,
    };
    use specs::WorldExt;
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
        let ecs_data = EcsActionMsgData::new(&entities, &players, &names);
        let player_entity = get_player_unwrap(&gs.ecs, PLAYER_NAME);
        let msg_out1 = entity_action_msg_no_ecs!(
            (&entities, &players, &names),
            "<SUBJ> {} the apple.",
            player_entity,
            "eat"
        );

        assert_eq!(msg_out1, "You eat the apple.");
        let msg_out2 = entity_action_msg!(&gs.ecs, "<SUBJ> {} the apple.", monster_entity, "eat");
        assert_eq!(msg_out2, "The goblin eats the apple.");
    }
}
