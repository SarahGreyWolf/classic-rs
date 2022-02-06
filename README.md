# ARCHIVED

I've decided that I'm dropping this project entirely and archiving it.\
I just lost all drive for it and the code I've written is a mess that even I struggle to wrap my head around now.\
It is technically usable right now though it has some issues due to clients not being multi-threaded such as the network thread being held up when a new player joins.\
You can give the feature/threaded-clients branch a go if you want that but in it's current state players get no interactions from other players.

# Classic-RS

A (W.I.P) implementation of the minecraft classic server.\
Aimed at supporting the project [Mineonline](http://mineonline.codie.gg/).

![Image of Classic.RS written in game in Trans Colours](https://github.com/Master0r0/classic-mc-rs/raw/dev/screenshots/splash.png)

## Current Implemented
- [X] Mojang Heartbeat
- [X] Mineonline Heartbeat
- [X] Packets
    - [X] ServerBound Packets
        - [X] Player Ident
        - [X] Set Block
        - [X] Position & Orientation
        - [X] Message
    - [X] ClientBound Packets
        - [X] Server Ident
        - [X] Ping
        - [X] Level Initialize
        - [X] Level Data Chunk
        - [X] Level Finalize
        - [X] Set Block
        - [X] Spawn Player
        - [X] Player Teleport
        - [X] Position and Orientation Update
        - [X] Position Update
        - [X] Orientation Update
        - [X] Despawn Player
        - [X] Message
        - [X] Disconnect Player
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
