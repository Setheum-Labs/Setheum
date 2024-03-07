# To-Do List

This list contains all TODOs in the Repo


<!-- TOC -->
- [ToDo List - The Monofile for Setheum Repo ToDos](#to-do-list)
  - [1. Introduction](#1-guidelines)
  - [2. Contribution](#2-contribution)
  - [3. Lists](#3-lists)
  - [4. Tasks](#3-tasks)
<!-- /TOC -->


## 1. Guidelines

Note: Before you write a ToDo in this repo, please read the below guidelines carefully.

Whenever you write a ToDo, you need to follow this standard syntax

```rust
//TODO:[file_name:task_number] - task_details
```

for example:

```rust
//TODO:[TODO.md:0] - Add Todo Guidelines
```

Note > the  `//TODO:[filename:task_number] - ` is what we call the `task_prefix`.

Whenever adding/writing a Task/ToDo, you need to describe the task on this list. Whenever you write a TODO in any file, add a reference to it here. Please make sure the task reference here is titled correctly and as detailed as possible\.

Whenever you `complete` a task/TODO from any file, please tick/complete its reference here and make sure you do it in the same `commit` that completes the task.

Whenever a task is cancelled (discontinued or not needed for w/e reason), please note in the details why it is cancelled, make sure you do it in the same `commit` that removes/cancels the TODO, and add this `-C` as a suffix to its `file_name` in the list here, for example:

```rust
//TODO:[TODO.md-C:0] - Add Todo Guidelines
```

## 2. Contribution

You can contribute to this list by completing tasks or by adding tasks(TODOs) that are currently in the repo but not on the list. You can also contribute by updating old tasks to the new Standard.

## 3. Lists

Each package/module/directory has its own `TODO.md`.

## 4. Tasks

These tasks are just for this file specifically.

- [x] [[TODO.md:0] - Add TODO.md File](TODO.md): Add a TODO.md file to organise TODOs in the repo.
- [x] [[TODO.md:1] - Add a `task_title`](/TODO.md/#tasks): Adda `task_title`.
- [ ] [[src/lib.rs:0] - Remove this from this module and add it to `EcdpSetrLoans` module.](/blockchain/modules/ecdp-ussd-loans/src/lib.rs): Add `OnLoanUpdate` to [EcdpSetrLoans Module](/blockchain/modules/ecdp-setr-loans/).
