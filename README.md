# RMP Example

A proof-of-concept demonstrating that the [RMP Architecture Bible](rmp-architecture-bible.md) is sufficient for any coding agent to build a fully working multi-platform app from scratch.

## What's Here

- **`rmp-architecture-bible.md`** -- The comprehensive guide for building Rust Multi-Platform apps. Covers philosophy, architecture (TEA/Elm), FFI (UniFFI), platform layers (SwiftUI, Jetpack Compose, iced), build pipelines, and patterns.
- **`rmp-cli/`** -- The `rmp` CLI tool that scaffolds new RMP projects with all platforms wired up and ready to build.
- **`hello-chat/`** -- A working "hello world" chat app (think Signal/iMessage) built entirely by a coding agent using only the bible as guidance. Targets iOS, Android, and Desktop (macOS/Linux).

## The Experiment

The goal: prove that the architecture bible alone is enough for an AI agent to go from zero to a working multi-platform app. The agent:

1. Read the bible
2. Used `rmp init` to scaffold the project
3. Replaced the starter code with a chat app (conversation list, message thread, compose + send)
4. Built and verified on macOS (iced desktop) and iOS (SwiftUI via xcodebuild)
5. Produced what it believes to be working Android code (Jetpack Compose) -- not run, but structurally correct per the bible

The entire process -- from reading the doc to a running app on two platforms -- was done in a single session.

## Hello Chat App

The chat app demonstrates the core RMP patterns:

- **Rust core** (`hello-chat/rust/`) -- `AppState`, `AppAction`, `AppUpdate`, `FfiApp` actor with flume channels. Mock data for 5 conversations with message histories. All business logic in Rust.
- **iOS** (`hello-chat/ios/`) -- SwiftUI with `@Observable` AppManager, NavigationStack driven by Rust Router, conversation list + chat detail + compose bar.
- **Android** (`hello-chat/android/`) -- Jetpack Compose with `mutableStateOf` AppManager, AnimatedContent navigation, Material 3 design.
- **Desktop** (`hello-chat/desktop/iced/`) -- iced 0.14 with two-panel layout (sidebar + chat), dark theme, Subscription-based state sync.

## Quick Start

```bash
cd hello-chat
nix develop    # sets up Rust, Android SDK/NDK, xcodegen, etc.

# Desktop (runs immediately):
just run-iced

# iOS (full pipeline):
just ios-full

# Android (requires cargo-ndk):
just android-full
```

## RMP CLI

```bash
# Scaffold a new project:
cargo run --manifest-path rmp-cli/Cargo.toml -- init my-app --org com.example --ios --android --iced --flake

# Inside a scaffolded project:
rmp doctor        # check prerequisites
rmp bindings all  # generate UniFFI bindings
rmp run ios       # build + run on iOS simulator
rmp run android   # build + run on Android emulator
rmp run iced      # run desktop app
```
