//! GENERATED CODE - DO NOT MODIFY

use cards_poc::{
    artifacts, champion_identities, champion_spells, minions, overlord_identities, overlord_spells,
    projects, schemes, weapons,
};
use rules::DEFINITIONS;

pub fn initialize() {
    DEFINITIONS.insert(artifacts::lodestone);
    DEFINITIONS.insert(artifacts::invisibility_ring);
    DEFINITIONS.insert(artifacts::accumulator);
    DEFINITIONS.insert(artifacts::mage_gloves);
    DEFINITIONS.insert(artifacts::skys_reach);
    DEFINITIONS.insert(artifacts::magical_resonator);
    DEFINITIONS.insert(artifacts::dark_grimoire);
    DEFINITIONS.insert(champion_identities::no_identity_law);
    DEFINITIONS.insert(champion_identities::no_identity_shadow);
    DEFINITIONS.insert(champion_identities::no_identity_primal);
    DEFINITIONS.insert(champion_spells::arcane_recovery);
    DEFINITIONS.insert(champion_spells::meditation);
    DEFINITIONS.insert(champion_spells::coup_de_grace);
    DEFINITIONS.insert(champion_spells::charged_strike);
    DEFINITIONS.insert(champion_spells::stealth_mission);
    DEFINITIONS.insert(champion_spells::preparation);
    DEFINITIONS.insert(minions::time_golem);
    DEFINITIONS.insert(minions::temporal_stalker);
    DEFINITIONS.insert(minions::shadow_lurker);
    DEFINITIONS.insert(minions::sphinx_of_winters_breath);
    DEFINITIONS.insert(minions::bridge_troll);
    DEFINITIONS.insert(minions::stormcaller);
    DEFINITIONS.insert(minions::fire_goblin);
    DEFINITIONS.insert(overlord_identities::no_identity_law);
    DEFINITIONS.insert(overlord_identities::no_identity_shadow);
    DEFINITIONS.insert(overlord_identities::no_identity_primal);
    DEFINITIONS.insert(overlord_spells::gathering_dark);
    DEFINITIONS.insert(overlord_spells::overwhelming_power);
    DEFINITIONS.insert(overlord_spells::forced_march);
    DEFINITIONS.insert(projects::gemcarver);
    DEFINITIONS.insert(projects::coinery);
    DEFINITIONS.insert(projects::spike_trap);
    DEFINITIONS.insert(schemes::gold_mine);
    DEFINITIONS.insert(schemes::activate_reinforcements);
    DEFINITIONS.insert(schemes::research_project);
    DEFINITIONS.insert(cards_test::test_overlord_identity);
    DEFINITIONS.insert(cards_test::test_champion_identity);
    DEFINITIONS.insert(cards_test::test_overlord_spell);
    DEFINITIONS.insert(cards_test::test_champion_spell);
    DEFINITIONS.insert(cards_test::test_scheme_31);
    DEFINITIONS.insert(cards_test::test_project_2_cost);
    DEFINITIONS.insert(cards_test::test_minion_end_raid);
    DEFINITIONS.insert(cards_test::test_minion_shield_1);
    DEFINITIONS.insert(cards_test::test_minion_shield_2_abyssal);
    DEFINITIONS.insert(cards_test::test_minion_deal_damage);
    DEFINITIONS.insert(cards_test::test_minion_infernal);
    DEFINITIONS.insert(cards_test::test_minion_abyssal);
    DEFINITIONS.insert(cards_test::test_minion_mortal);
    DEFINITIONS.insert(cards_test::test_weapon_2_attack);
    DEFINITIONS.insert(cards_test::test_weapon_2_attack_12_boost);
    DEFINITIONS.insert(cards_test::test_weapon_3_attack_12_boost);
    DEFINITIONS.insert(cards_test::test_weapon_4_attack_12_boost);
    DEFINITIONS.insert(cards_test::test_weapon_abyssal);
    DEFINITIONS.insert(cards_test::test_weapon_infernal);
    DEFINITIONS.insert(cards_test::test_weapon_mortal);
    DEFINITIONS.insert(cards_test::test_weapon_5_attack);
    DEFINITIONS.insert(cards_test::activated_ability_take_mana);
    DEFINITIONS.insert(cards_test::triggered_ability_take_mana);
    DEFINITIONS.insert(cards_test::test_0_cost_champion_spell);
    DEFINITIONS.insert(cards_test::test_1_cost_champion_spell);
    DEFINITIONS.insert(cards_test::deal_damage_end_raid);
    DEFINITIONS.insert(cards_test::test_card_stored_mana);
    DEFINITIONS.insert(cards_test::test_attack_weapon);
    DEFINITIONS.insert(weapons::marauders_axe);
    DEFINITIONS.insert(weapons::keen_halberd);
    DEFINITIONS.insert(weapons::ethereal_blade);
    DEFINITIONS.insert(weapons::bow_of_the_alliance);
}
