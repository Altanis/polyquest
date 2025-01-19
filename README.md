# poly quest

each player has a host planet. there exist three other entities:
- planets (ATTACK for exp)
- comets (collide with for health, attacking gives extra health)
- stars (collide with for energy)

all have their own health. attacking does more dmg than collision, and killing one gives you some exp.

planets act as bases where players are invincible, but leaving a planet leaves it susceptible to attack. if the planet dies, itll take 5min to regenerate (leaving player with no base). players cant attack if theyre on their own planet.

planets: resembles earth. glows red if its not yours, glows green if it is yours.
stars: according to hr diagram (blue, cyan, green, yellow, orange, red, where blue is highest energy and red is lowest)
comets: polygonal shape with cometary ectoplasmic thing around it

--- 
Todo:
- [x] align tank upgrades correctly
- [x] fix hovering occurring everywhere
- [x] fix progress bars not being accurate
- [x] fix no re-rendering when hovering over ui element
- [ ] soundtrack for ingame fails
- [ ] bound drones to walls
- [ ] required name field
- [ ] continue button needs to be moved along to click
- [ ] close button for tank upgrades
- [ ] audio tracks interfere cuz theyre on the dom
- [ ] hovering tooltip
- [ ] minimap
- [ ] mspt counter
- [ ] leaderboard
- [ ] leader arrow
- [ ] finish tanks
- [ ] clans
- [ ] global + local chat (or maybe clan-specific chats)
- [ ] keybinds for stat upgrades
- [ ] on death, make spectate button which clears ui and allows u to roam

Ideas:
- tank ideas:
    - impulse: bullets which target nearest enemy
    - sinusod: fires two bullets which vary according to sine/cosine paths