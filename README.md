# Sysadmin - A Shell for Sysadmins

> "I'm spending my afternoon as `#!/usr/bin/sysadmin`" - a human shell script

## What is this?

`sysadmin` is an interactive executor for operational documentation. It's a pun: a shell for sysadmins, where *you* are the shell.

At many organizations, we maintain small scripts that are part documentation, part human runbook. This tool provides a shell to facilitate the "running" of these documents, with the human operator as the interpreter.

## File Format

`.sysadmin` files are markdown documents with executable code blocks:

```bash
#!/usr/bin/sysadmin

# Database Migration

This guides you through the migration process.

## Step 1: Verify backup

Check that backups are recent:

```bash
ls -lh /var/backups/db/
```

## Step 2: Run migration

```bash
./migrate.sh --env production
```

All done!
```

## Installation

```bash
cargo build --release
sudo cp target/release/sysadmin /usr/bin/sysadmin
```

Make your `.sysadmin` files executable:

```bash
chmod +x my-runbook.sysadmin
```

## Usage

### Interactive Mode (default)

```bash
sysadmin my-runbook.sysadmin
# or if executable:
./my-runbook.sysadmin
```

This will:
1. Display documentation and context
2. Show each code block with syntax highlighting
3. Pause and wait for you to run the command
4. Continue to the next step when you press Enter

### Dry Run

Preview all steps without executing:

```bash
sysadmin dry-run my-runbook.sysadmin
```

### View

Display the file as plain documentation:

```bash
sysadmin view my-runbook.sysadmin
```

## Features

- ✅ Markdown-based format (familiar and readable)
- ✅ Syntax highlighting for code blocks
- ✅ Interactive step-through execution
- ✅ Support for multiple languages (bash, python, ruby, etc.)
- ✅ Clean, colorful terminal output
- ✅ Shebang support (`#!/usr/bin/sysadmin`)

## Use Cases

- Database migrations
- Production deployments
- Incident response procedures
- Manual failover procedures
- Maintenance tasks
- Any operational task requiring human judgment

## Development

### Project Structure

```
sysadmin/
├── src/
│   ├── main.rs          # CLI entry point
│   ├── lib.rs           # Library exports
│   ├── model/           # Document data structures
│   ├── parser/          # Markdown parser
│   ├── executor/        # Interactive execution
│   ├── ui/              # Terminal rendering
│   └── cli.rs           # CLI argument parsing
└── examples/            # Example .sysadmin files
```

### Running Tests

```bash
cargo test
```

### Running Examples

```bash
cargo run -- examples/basic.sysadmin
cargo run -- dry-run examples/database-migration.sysadmin
```

## Future Ideas

See [SPECIFICATION.md](./SPECIFICATION.md) for planned features:

- Task labels and checkpoints
- Variable substitution
- Safety annotations (`:confirm`, `:danger`)
- Structured logging
- Enhanced TUI with split-screen shell
- Conditional execution
- And more!

## License

MIT OR Apache-2.0
