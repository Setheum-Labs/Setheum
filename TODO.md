# ToDo List - The Monofile for Setheum Repo ToDos

This list contains all TODOs in the Repo


<!-- TOC -->
- [ToDo List - The Monofile for Setheum Repo ToDos](#setheum---the-monofile-for-setheum-repo-todos)
  - [1. Introduction](#1-guidelines)
  - [2. Contribution](#2-contribution)
  - [3. Tasks](#3-tasks)
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

Whenever adding/writing a Task/ToDo, you need to open an [Issue](https://github.com/Setheum-Labs/Setheum/issues) which references the commit that adds/writes the task or vice-versa. The `Issue Title` should be the `task_prefix` andn the `task_details` altogether just as it is written in the TODO.

Whenever you write a TODO in any file, please add a reference to it here. Please make sure to title the task reference here exactly as the `task_prefix` (excluding the `task_details`).

Whenever you `complete` a task/TODO from any file, please tick/complete its reference here and add a reference to the `commit` that completes the task.

Whenever a task is cancelled (discontinued or not needed for w/e reason), add this `-C` as a suffix to its `file_name`, for example:

```rust
//TODO:[TODO.md-C:0] - Add Todo Guidelines
```

Note > The suffix need not be added to the reference Issue too, but you may if you want.


## 2. Contribution

You can contribute to this list by completing tasks or by adding tasks(TODOs) that are currently in the repo but not on the list.
To search for a todo, you can search for TODOs based on the `task_prefix`. You can also contribute by updating old tasks to the new standard syntax using `task_prefix`, you can also open issues for those Tasks if not opened before, if opened before you can link to those Issues as the reference.

## 3. Tasks

- [x] [Add TODO.md File](TODO.md)
- [x] [Categorize the Tasks with a `task_prefix`](/TODO.md/#tasks)
