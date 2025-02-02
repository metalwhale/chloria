## Package code structure
Each package in this workspace follows a code structure inspired by [the Clean Architecture](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html) with some slight modifications:
- `domain` module (enterprise business rules): contains entities
- `execution` module (application business rules): contains a workshop, use cases ("cases" for short), ports and tasks
- `interface` module: contains adapters for external frameworks, such as controllers (which handle requests) and presenters (which return responses)
- `infrastructure` module: contains clients and tools for external services, such as cloud providers, third-party API platforms, drivers...

```
                         ↓ ↑
┌───────────────────────────────────────────────────┐
│                     Interface                     │
└───────────────────────────────────────────────────┘
     ↓ ↑                 ↓ ↑                 ↓ ↑
    case                case                case
┌───────────────────────────────────────────────────┐
│                Execution / Domain                 │
├───────────────────────────────────────────────────┤
│                     Workshop                      │
│   task                task                task    │
└───────────────────────────────────────────────────┘
    port                port                port
     ↓ ↑                 ↓ ↑                 ↓ ↑
┌───────────────────────────────────────────────────┐
│                  Infrastructure                   │
└───────────────────────────────────────────────────┘
```
- These modules follow the dependency rule:
  - Code dependencies (within a package) from `interface` proceed as: `interface` -> `execution` -> `domain`
  - Similarly, code dependencies from `infrastructure` proceed as: `infrastructure` -> `execution` -> `domain`
  - There are no code dependencies between `interface` and `infrastructure`: they are on the same level, connecting directly to `execution` without relying on or knowing about each other
  - Glue code that connects all the modules must be placed at the outermost scope (i.e. `main` function)
- The data flows in this order: `interface` -> `execution / domain` -> `infrastructure`:
  - `interface` calls `execution`'s use cases, and in turn `execution` calls `infrastructure`'s services (which implement `execution`'s ports); all of these calls use DTOs as parameters and return values, without needing to reference `entity`
  - `interface` is only aware of workshop and use cases, while `infrastructure` is only aware of ports
  - In contrast, neither `interface` nor `infrastructure` is aware of tasks, as tasks should only be performed internally within `execution`
