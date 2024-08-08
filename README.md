## Grasp Primary Module (GPM)
This repository contains the source code for the primary software module of the Grasp project. This project is still very much in its early stages, but the goal is to create a highly extensible, asynchronous framework for building embedded systems on the Raspberry Pi.

## Purpose
The purpose of the GPM is to manage all systems of the arm. It essentially serves as an asynchronous, non-blocking task scheduler. Some example tasks include BMS monitoring, EMG processing, servo motor controls and overall system health monitoring. It is based on Tokio's "task" asbtraction (a.k.a green threads). This design decision was made to avoid the overhead of OS-level threads given that GPM runs on the single-core Raspberry Pi Zero.

## Installation
```bash
# After cloning the repository, run the following command to initialize the `sgcp` submodule
git submodule init
git submodule update

# Install rust on your system by following these instructions: https://www.rust-lang.org/tools/install
cd <REPOSITORY>
cargo run
```

## System Overview
GPM and Analytics make up the embedded software for GRASP. While GPM is at the heart of the arm and is responsible for interfacing to every component, the Analytics module is the "brain" of the arm and is responsible for making the decisions on grip type based on EMG readings from the pilot and the camera feed. The GRM (Grasp Remote Module) component hosts a Prometheus server that scrapes data from GPM and displays a Grafana dashboard for monitoring.


<img width="705" alt="image" src="https://github.com/BEARUBC/gpm/assets/83952444/93593ce3-ae5c-4ff7-b5b1-715653146154">

## File Structure
TODO!

## Next Steps
TODO!
