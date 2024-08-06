# Nix Service Manager

A daemon that can manage services declaratively using nix.

## Project Goals

- [ ] Declarative service configuration with Nix.
- [ ] Automatic service restart when the system reboots.
- [ ] Continuous Deployment from any git repository (not just GitHub).
- [ ] Simple to install, setup, and use

## System Requirements

You will need to have the experimental `nix-command` feature turned on. See [the Wiki](https://nixos.wiki/wiki/Nix_command) for more info.

## Motivation

I use NixOS on my server and I was looking for a way to get declarative Continuous Deployment. Hydra is good for Continuous Integration but it does not do Continuous Deployment. Thus, I set out to create this project to fufill my needs.
