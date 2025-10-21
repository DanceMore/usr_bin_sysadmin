# Sysadmin File Format Specification v0.1

## Origin Story

The name comes from a joke: "I'm spending my afternoon as `#!/usr/bin/sysadmin`" - a human shell script.

Just as `/bin/sh` interprets shell scripts, `/usr/bin/sysadmin` interprets humans executing operational tasks. At many organizations, we maintain small scripts that are part documentation, part human runbook. This tool provides a shell to facilitate the "running" of these documents, with the human as the interpreter.

It's a pun: a shell for sysadmins, where *you* are the shell.

## Overview

Sysadmin files are executable documentation for operational tasks that require human judgment, supervision, or cannot be fully automated. The format is designed to be read and executed interactively, with a human operator reading context, executing commands, and progressing through steps.

## Design Goals

- **Human-readable**: Valid sysadmin files should be pleasant to read as plain text documentation
- **Shell-familiar**: Leverage existing shell script conventions
- **Interactive execution**: Read step, run step, observe output, continue
- **Simple first**: Start minimal, grow features as needed

## File Format

### Extension
`.sysadmin`

### Shebang
```
#!/usr/bin/sysadmin
```

Files are directly executable if the `sysadmin` binary is installed at `/usr/bin/sysadmin`.

### Basic Structure

```sysadmin
#!/usr/bin/sysadmin

# Database Migration - Q4 2025

This file guides you through migrating the production database
to the new schema version 4.2.

## Prerequisites

Before starting, ensure:
- You have production database credentials
- A tested backup exists from the last hour
- The maintenance window has started

## Steps

### Verify backup exists

Check that the automated backup completed successfully:

```bash
ssh backuphost 'ls -lh /var/backups/db/latest.sql.gz'
```

### Stop application servers

This prevents new writes during migration:

```bash
kubectl scale deployment/api-server --replicas=0
```

### Run migration

```bash
psql -h proddb.internal -U dbadmin -f migration-v4.2.sql
```

You should see output indicating successful schema changes.

### Verify migration

```bash
psql -h proddb.internal -U dbadmin -c "SELECT version FROM schema_migrations ORDER BY version DESC LIMIT 1;"
```

Expected output: `4.2.0`

### Restart application

```bash
kubectl scale deployment/api-server --replicas=5
```

## Rollback

If issues occur, run:

```bash
psql -h proddb.internal -U dbadmin -f rollback-v4.2.sql
```
```

## Syntax Elements

### Comments / Documentation
Lines starting with `#` are documentation. All Markdown syntax is supported for formatting.

### Code Blocks

Executable commands are placed in fenced code blocks with a language identifier:

````
```bash
command-to-run
```
````

Other language identifiers like `sh`, `python`, `ruby` are also valid and will be executed with the appropriate interpreter.

Code blocks without a language identifier are treated as documentation.

### Task Sections

Optional markdown headers (any level) can be used to organize steps:

```
### Verify backup exists
```

These provide structure and context but don't affect execution.

## Execution Model

### Interactive Mode (Primary)

The default execution model is:

1. **Read**: Display documentation and context (all markdown/comments)
2. **Execute**: Show the next code block with syntax highlighting
3. **Pause**: Wait for operator to run the command manually
4. **Continue**: Operator presses Ctrl-D (or confirms) to proceed to next step
5. **Repeat**: Continue through all steps in the file

The human operator is the interpreter - they read, understand context, execute commands in their own shell, observe output, and decide when to continue.

### Modes

1. **Interactive** (default): Step through each code block with operator confirmation
2. **Dry-run**: Display all tasks in sequence without pausing for execution
3. **View**: Render the file as formatted documentation (no execution prompts)

## Example Use Cases

- Database migrations
- Production deployments  
- Incident response procedures
- Backup restoration
- Manual failover procedures
- Vendor escalation workflows
- Security incident response
- Any operational task requiring human judgment

## Future Considerations

### Potential Features
- **YAML front matter**: Metadata like title, author, version, tags
- **Task labels**: Named checkpoints (e.g., `### task:verify-backup ###`)
- **Variables**: Template substitution like `${HOST:default.value}`
- **Safety annotations**: `:confirm`, `:danger` levels on code blocks
- **Manual task blocks**: For non-command actions (UI clicks, phone calls)
- **Expected output validation**: `@expect: "success"` annotations
- **Timeouts and retries**: `@timeout: 5m`, `@retry: 3`
- **Conditionals**: Execute blocks based on previous step results
- **Logging**: Structured execution logs (JSON/YAML)
- **Resume capability**: Continue from a checkpoint
- **Parallel execution**: Independent tasks run concurrently
- **Remote execution**: Run commands on specified hosts
- **Include/import**: Compose larger runbooks from smaller ones
- **Web UI**: Visual progress tracking and collaboration
- **Integration**: Ticketing systems, chat ops, audit systems

### Implementation Ideas
- **TUI enhancements**: 
  - Split screen: documentation + embedded shell
  - Syntax highlighting for code blocks
  - Progress breadcrumbs
  - Searchable history of completed steps
- **Shell integration**: 
  - Drop into a sub-shell while keeping runbook visible
  - Shell command history includes runbook context
- **Colorful output**: 
  - Distinct colors for documentation vs commands
  - Status indicators (pending, running, complete)
  - Syntax highlighting for various languages
