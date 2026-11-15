# CATastrophe input backend

This Backend rust and html Project is handling the Phone input and managing the communication between the parties. 
THe Goal of the backend is to provide a input method every one can join to by QR code 

```
cargo run --release          # PORT=8080, TICK_HZ=30 by default
```


Phones must be on the same network. On Windows, allow the binary through the firewall on first start.

## Pages
- `GET /` – display/lobby page: QR code, live input visualisation, scoreboard
- `GET /controller` – phone joystick page (the QR target)
- `GET /qr.svg` – just the QR code

## Phone → server: `WS /ws/controller?name=<opt>&id=<opt>`
Server sends `{"type":"welcome","id":1,"name":"Cat 1","color":"#36e268"}`, then a `status` message every 500 ms.
Phone sends its stick as `[x, y]` or `{"x":..,"y":..}`, each in -1..1 (y up = positive). Values are clamped into the unit circle.
`id` resumes a disconnected player (and their score); the controller page does this automatically via localStorage.

## Server → game: `WS /ws/game`
Every tick:
```json
{"type":"tick","tick":87,"x":0.9,"y":0.1,"magnitude":0.905,"activePlayers":2,"connectedPlayers":2,
 "players":[{"id":1,"x":1,"y":0},{"id":2,"x":0.8,"y":0.2}]}
```
`x`/`y` = the combined ("normed") input: the average of all players whose stick is outside the 0.1 dead zone, always inside the unit circle.
About 4x per second the same socket also gets a `scores` message (same format as `/api/scores`).
The game can send `{"type":"resetScores"}`.

Polling alternative: `GET /api/input` returns the latest tick.

## Scores: `GET /api/scores`, `POST /api/scores/reset`
```json
{"type":"scores","players":[{"rank":1,"id":1,"name":"Alice","color":"#36e268","connected":true,
  "accuracy":0.93,"averageAccuracy":0.93,"score":1.12}]}
```
- `accuracy` (0..1): on the latest tick, `1 - |player - combined| / 2`. 1 = exactly on the combined input, 0 = opposite edge. `null` while the player and the group are both idle.
- `averageAccuracy`: average over all scored time.
- `score`: accumulated accuracy × seconds, i.e. seconds of perfect agreement. Sorted descending.

Standing still while the group moves still counts (about 0.5), so idling does not pay off.
