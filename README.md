# Classic-RS
A (W.I.P) implementation of the minecraft classic server.\
Aimed at supporting the project [Mineonline](http://mineonline.codie.gg/).

## Current Implemented
- [x] Mojang Heartbeat
- [x] Mineonline Heartbeat
- [ ] Packets
    - [X] ServerBound Packets
        - [X] Player Ident
        - [X] Set Block
        - [X] Position & Orientation
        - [X] Message
    - [ ] ClientBound Packets
        - [X] Server Ident
        - [X] Ping
        - [X] Level Initialize
        - [X] Level Data Chunk
        - [X] Level Finalize
        - [X] Set Block
        - [ ] Spawn Player (Spawns new player for others but not others for new player)
        - [X] Player Teleport
        - [X] Position and Orientation Update
        - [X] Position Update
        - [X] Orientation Update
        - [ ] Despawn Player
        - [X] Message
        - [X] Disconnect Player
        - [ ] Update User Type
- [ ] World
    - [ ] [ClassicWorld Format](https://wiki.vg/ClassicWorld_file_format)
        - [ ] Loading
        - [ ] Saving
        - [X] Loading as CRS Binary
        - [X] Saving as CRS Binary
        - [X] Creation (A flat world)
    - [ ] Classic DAT Format
        - [ ] Loading
        - [ ] Saving
        - [ ] Creation
- [ ] Console
    - [ ] Input
    - [ ] Fancy Stuff
- [ ] Plugin System