This damage formula might not be 100% accurate, but it should be very close.

```rust
struct Attacker {
    hp: f64,

    // Base damage that the unit deals to the other unit
    base_damage: f64,

    // Attack bonuses like Max directs
    co_bonus: f64,

    // Attack penalties like Max indirects
    co_penalty: f64,

    // Whether the CO power is active or not
    co_power: bool,

    // Number of comtowers
    comtowers: f64,

    // Defaults to 9%
    good_luck: f64,

    // Defaults to 0%
    bad_luck: f64,
}

impl Attacker {
    fn co_bonus(&self) -> f64 {
        // All COs get a +10% attack during their CO power
        if self.co_power {
            self.co_bonus - self.co_penalty + 0.1

        } else {
            self.co_bonus - self.co_penalty
        }
    }

    fn damage(&self) -> f64 {
        // 10% attack bonus per comtower
        let bonus_damage = self.co_bonus() + (self.comtowers * 0.1);

        // Minimum 1% damage
        let attack_damage = (self.base_damage * bonus_damage).min(0.01);

        let luck = random(0.0, self.good_luck) - random(0.0, self.bad_luck);

        // Attack damage and luck are scaled by the unit's HP
        self.hp.ceil() * (attack_damage + luck)
    }
}


struct Defender {
    hp: f64,

    // Defense bonuses like Kanbei
    co_bonus: f64,

    // Defense penalties like Grimm
    co_penalty: f64,

    // For Javier only
    comtowers: f64,

    // 10% for each terrain star
    terrain_stars: f64,
}

impl Defender {
    fn defense(&self) -> f64 {
        // 10% defense bonus per terrain star, but scaled by HP.
        // Terrain bonus is doubled during Lash's SCOP but it still scales with HP.
        let terrain_defense = self.hp.ceil() * (self.terrain_stars * 0.1);

        // 10% defense bonus per comtower but only for Javier
        let comtowers = self.comtowers * 0.1;

        let bonus_defense = self.co_bonus - self.co_penalty + comtowers;

        bonus_defense + terrain_defense
    }
}


fn calculate_damage(attacker: &Attacker, defender: &Defender) -> f64 {
    (attacker.damage() * defender.defense()).trunc()
}
```
