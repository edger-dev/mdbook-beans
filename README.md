# mdbook-beans

An [mdBook](https://rust-lang.github.io/mdBook/) preprocessor that injects [beans](https://github.com/hmans/beans) task data into your book — turning documentation into a project dashboard.

## What it does

`mdbook-beans` reads your project's `.beans/` markdown files and generates two chapters in your book:

- **Kanban** — A board view with status columns (Todo, In Progress, Done), excluding drafts and archived tasks. Epics show subtask progress badges; subtasks indicate their parent epic.
- **All Tasks** — A structured reference of every bean with stable URLs (`/beans/<bean-id>`), organized by type (Epics, Features, Tasks, Bugs, Drafts).

No beans runtime is required — the preprocessor reads the markdown files and `.beans.yml` config directly.

## Usage

Add the preprocessor to your `book.toml`:

```toml
[preprocessor.beans]
```

Place markers in your `SUMMARY.md` where you want the chapters to appear:

```markdown
# Summary

- [Introduction](./intro.md)
- [Kanban]({{#beans-kanban}})
- [All Tasks]({{#beans-tasks}})
```

## Requirements

- A `.beans.yml` configuration file at the project root
- Bean markdown files in the configured beans directory (default `.beans/`)

## License

MIT
