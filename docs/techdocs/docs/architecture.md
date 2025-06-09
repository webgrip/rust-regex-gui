# Architecture

Regex GUI follows a simple hexagonal architecture inspired by the works of Kent Beck, Robert C. Martin, Sam Newman and Eric Evans. The main layers are:

- **Domain**: core business types and logic (e.g. `Rule`).
- **Application**: orchestrates use cases (`Renamer`).
- **Infrastructure/UI**: user interface and external concerns (e.g. telemetry).

```mermaid
%%{init: {'theme':'base'}}%%
%% include the external mermaid file
```

![Architecture](diagrams/architecture.mmd)
