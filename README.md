# bevy-context-steering

A 3D (and 2D) agent context steering framework for Bevy + Avian3D.

## What it is

Each agent gets a set of sampled directions — a Fibonacci sphere in 3D, or a plane in 2D — with the number and layout of directions configurable per agent. Behaviours don't pick a single output direction directly; they vote into per-direction **interest** (worth moving toward) and **danger** (avoid) maps, plus a **velocity** map. These get resolved into a target direction and target velocity, which motion then acts on.

## Install

```bash
cargo add bevy-context-steering
```

## Features

- `debug` — gates `DebugSteeringPlugin` and the interest/danger/velocity visualization.

## Setup

Add the plugins:

```rust
app.add_plugins((SteeringPlugin, DebugSteeringPlugin));
```

`DebugSteeringPlugin` is optional — gives you visualization of the interest/danger/velocity maps per agent.

Mark anything that should steer with `Agent`:

```rust
commands.spawn((Agent, /* ... */));
```

Behaviours only apply to entities carrying `Agent`.

## Behaviours

- **Normal** — single-target behaviours (Seek, Flee, Evade, ...). Need a target position/point.
- **Neighbour** — immediate-area behaviours (Cohere, Separate, Align). React to nearby agents directly.
- **Cluster** — behaviours operating over logical groupings of agents rather than individual neighbours.
- **Obstacle** — avoidance behaviours reacting to obstacle geometry.

## Motion

Common motion types are implemented out of the box, but movement is customizable — build your own on top of the resultant direction and resultant velocity if an agent needs bespoke motion (e.g. rail-constrained, fixed-height hover, etc.).

## Examples

Normal:

```rust
commands.spawn((Agent, Seek::new(target), /* ... */));
```

Neighbour and obstacle behaviours are added the same way:

```rust
commands.spawn((Agent, Cohere::new(), /* ... */));
commands.spawn((Agent, AvoidObstacles::new(), /* ... */));
```

Cluster behaviours are also added the same way, but if the agent should belong to a cluster, insert enter/exit-cluster components (otherwise it behaves like any other agent):

```rust
commands.spawn((Agent, CohereCluster::new(), /* ... */)).enter_cluster(cluster_id);
```

## Versions

| crate | bevy   | avian3d |
|-------|--------|---------|
| 0.1.0 | 0.19.0 | 0.7.0   |

## Example

```rust
// Normal
commands.spawn((Agent, Seek::new(target)));

// Neighbour
commands.spawn((Agent, Cohere::new(), Separate::new(), Align::new()));

// Obstacle
commands.spawn((Agent, ObstacleAvoidance::new()));

// Cluster — same as above, plus opt in/out of a cluster explicitly
commands.spawn((Agent, Cohere::new()));
commands.entity(agent).insert(EnterCluster(cluster_id));
// ...
commands.entity(agent).remove::<EnterCluster>(); // or an ExitCluster event, whichever you exposed
```

## License

Apache-2.0
