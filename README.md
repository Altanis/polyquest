## note

this project will likely go unmaintained for a few months due to a loss of motiviation.

---


# poly quest

## how to run

first install [cargo is installed](https://doc.rust-lang.org/cargo/getting-started/installation.html), [wasmpack is installed](https://rustwasm.github.io/wasm-pack/installer/), and [https://www.npmjs.com/package/serve](serve).

running this game is quite simple. run `./client_prod.sh` and `./server.sh`, then connect to `http://localhost:{port}`, where `{port}` is whatever the output of ./client.sh tells you the client is running on.

<!--[Stars]: Players can reside in stars to regain HP.
    -> Has a corona aligned with the color of the star.
    -> Has a few shapes in a gravitional orbit.
    -> Star colors follow the HR diagram based on its energy.
        -> Higher "energy" stars impart more health.
    -> Killing a star gives EXP and reduces the star's health.
        -> Stars can de-evolve based on their health.
        -> They regenerate automatically according to Diep.io regeneration mechanics.
    -> Star death leads to the corona flaring out, then a dull white core being formed.
        -> This core cannot be attacked.
        -> A timer is attached to it, signifying when it'll "rebirth".
        -> Rebirth randomizes energy level.

[Beacon]: Clans can contest beacons for passive benefits.
    -> There exists beacons in four quadrants (NESW).
    -> Similar to dominators, beacons can be contested.
        -> Global notifications on beacon ownership changes occur.
    -> Clans which own the beacon get passive benefits.

[Boss]: Bosses that have high HP.
    -> Has a collection of orbs around it.
         -> Upon death, an orb spawns back 30 seconds later.
         -> The orbs are sent to attack if the boss/one of its orbs are attacked.
    -> Bosses attack when player is in range of the boss.
    -> Travels around the map.
    -> Regenerates according to Diep.io regeneration mechanics.

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
- [x] finish stylistic effect of celestial orb
- [x] leader arrow does not conform to dpr
- [x] time not reset after respawn
- [x] enter to respawn
- [x] battleship shoots even when auto fire off
- [x] mouse in bounds check
- [x] zooming into page causes fov change
- [ ] finish tanks
- [ ] finish bodies
- [x] passive regen
    - [x] maybe add a bar for regeneration time?
- [x] local chat
- [ ] scrolling within modal
- [ ] clans
    - [ ] create clan system
    - [ ] join system
    - [ ] leave system
    - [ ] kick system
    - [ ] right/left arrows based on page
- [ ] fix leader arrow
- [x] menu items do not conform to dimensions/dpr
- [ ] find better audio tracks (stable audio)
- [ ] audio tracks interfere cuz theyre on the dom
- [ ] close button for tank upgrades
- [ ] keybinds for stat upgrades
- [ ] remove health regen as an upgrade
- [ ] on death, make spectate button which clears ui and allows u to roam
- [ ] laser tank branch
- [ ] tutorial screen
- [ ] saying slur -> delevels
- [ ] sounds
- [ ] auto fire when typing E in name tag

-- SHOULD BE DONE BY FEBRUARY BREAK ^^ --

then, make polyquest features:

Ideas:
- tank ideas:
    - impulse: bullets which target nearest enemy
    - sinusod: fires two bullets which vary according to sine/cosine paths
- passive exp gain (500xp/min)
- tutorial instead of lore--!>
