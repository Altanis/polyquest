# poly quest

Stars: Players can reside in stars to regain HP.
    -> Has a corona aligned with the color of the star.
    -> Star colors follow the HR diagram based on its energy.
        -> Higher "energy" stars impart more health.
    -> Attacking a star gives EXP and reduces the star's health.
        -> Stars can de-evolve based on their health.
        -> They regenerate automatically according to Diep.io regeneration mechanics.
    -> Star death leads to the corona flaring out, then a dull white core being formed.
        -> This core cannot be attacked.
        -> A timer is attached to it, signifying when it'll "rebirth".
        -> Rebirth randomizes energy level.

Orbs: Entities which give EXP upon death.
| **Orb Tier**              | **HP**| **EXP**| **Radius (px)**| **Notes** |
|---------------------------|-------|--------|----------------|---------------|
| **1 - Flickering Orb**    | 10    | 8      | **30 px**  | Smallest, low value |
| **2 - Basic Energy Orb**  | 30    | 20     | **55 px** | Early-game orb |
| **3 - Stable Orb**        | 100   | 110    | **65 px** | Medium-sized, efficient |
| **4 - Heavy Orb**         | 400   | 500    | **85 px** | Starts to feel large |
| **5 - Radiant Orb**       | 1500  | 2000   | **105 px** | Big, high-risk high-reward |
| **6 - Celestial Orb**     | 5000  | 7000   | **120 px** | Huge, rare, game-changing |

--- 
Todo:
- [x] align tank upgrades correctly
- [x] fix hovering occurring everywhere
- [x] fix progress bars not being accurate
- [x] fix no re-rendering when hovering over ui element
- [x] soundtrack for ingame fails
- [x] bound drones to walls
- [x] required name field
- [x] hovering tooltip
- [x] minimap
- [x] mspt counter / latency
- [x] leaderboard
- [x] leader arrow
    - [x] too small distance to make inviisble
- [x] ai targets tanks
- [x] killing drones fucks up projectile count
- [ ] finish tanks
- [ ] finish bodies
- [ ] mouse in bounds check
- [ ] clans
- [ ] global + local chat (or maybe clan-specific chats)
- [ ] fix leader arrow
- [ ] find better audio tracks (stable audio)
- [ ] audio tracks interfere cuz theyre on the dom
- [ ] close button for tank upgrades
- [ ] keybinds for stat upgrades
- [ ] remove health regen as an upgrade
- [ ] on death, make spectate button which clears ui and allows u to roam
- [ ] laser tank branch
- [ ] tutorial screen

-- SHOULD BE DONE BY FEBRUARY BREAK ^^ --

then, make polyquest features:

Ideas:
- tank ideas:
    - impulse: bullets which target nearest enemy
    - sinusod: fires two bullets which vary according to sine/cosine paths
- passive exp gain (500xp/min)
- tutorial instead of lore
