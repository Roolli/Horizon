# must have

- lightculling
- physics
- performance testing
- unit testing??
- 2 APPS - Marbles on stream + ????
- scripting extensions

## Light culling
- ~~calculate proper radius~~
- ~~utilize buffers in forward fragemnt shader~~
- figure out why it's not calculating proper frustums
~~- MAYBE do spot lights too.~~
  
## Physics
- ~~Calculate composite shapes for each object that has more than 1 mesh~~
- wire-up collision detection user events
- add colshapes
- wire-up colshape events

## Performance testing

Setup GPU execution time measurement with exported data for each frame.
Add CPU time measurement (maybe do a resource in ECS?)


### Create following test scenarios: 
- Scene with many lights  (sponza + moving lights) show off light culling with / without optimalization.
- Instancing show off with same objects being rendered per draw call vs as 1 draw call
- Physics show off (with marbles on stream)
- MAYBE if CSM is fixed show off performance vs detail.
- Scripting show off with second example and put many setIntervals & timeouts and measure impact on those??
  
## Unit/Integration testing

### Create tests for the following modules:
- Scripting / Conversions (Both for JS and rust side)
- Loading utils (+ make loading return error rather than crash)

## Apps

### Marbles on stream

- create / get some sort of slide
- get some balls (kekw)
- add physics
- first one to 'finish' wins
- ????
- profit
### Test suite
- setup test scenes using this

## Scripting extensions
- Add mouse / keyboard events 
- Hook up collision events
- Add method of creating colshapes 
- ~~Add attachment to camera and/or~~ lights
- if UI builder is done add events for closing / interacting with certain elements


# would be nice
- CSM fix
- PBR shader calculations
- UI builder
- async file loading???

## CSM Fix
- Check calculations

## PBR shaders

- use data in buffers to add basic pbr properties and image based lighting (no environment/ reflection mapping)

## UI Builder
- use json structure to desrcibe elements
- hook up events through some sort of id system
