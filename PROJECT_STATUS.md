# WOPlanner Project Status

## kabai Configuration

- **Project ID:** `15`
- **Project Slug:** `woplanner`
- **Project Name:** WOPlanner
- **Description:** A Progressive Web App for tracking workouts and monitoring strength progression. Built with Rust, HTMX, and SQLite.

## Board Workflow

```
Backlog → In Progress → Review → Done
     ↖___________/
```

### Status Overview

| Status | ID | Position | Agent Role Instruction |
|--------|-----|----------|------------------------|
| Backlog | 77 | 1 | Tickets im Backlog sind geplant, aber noch nicht in Arbeit. Bevor du mit einem Ticket beginnst, prüfe die verlinkten Dokumente und erstelle/aktualisiere fehlende ADRs oder Architekturentscheidungen. |
| In Progress | 78 | 2 | Aktuell in Arbeit. Stelle sicher, dass alle notwendigen Dokumente verlinkt sind und erstelle Tasks für jede Akzeptanzkriterie. |
| Review | 79 | 3 | Review-Phase. Prüfe Code-Qualität und verlinke alle erstellten Dokumente. |
| Done | 80 | 4 | Abgeschlossen. Das Ticket erfüllt alle Akzeptanzkriterien und ist bereit für Deployment. |
| Human Intervention | 75 | 98 | Dieses Ticket wartet auf menschliche Intervention. Lies alle Kommentare, beantworte die Frage des Agenten und verschiebe das Ticket danach nach "human_answered". |
| Human Answered | 76 | 99 | Der Mensch hat geantwortet. Lies die neuesten Kommentare und fahre mit der Arbeit fort. Verschiebe das Ticket in den passenden Folgestatus. |

### Allowed Transitions

- Backlog (77) → In Progress (78)
- In Progress (78) → Backlog (77)
- In Progress (78) → Review (79)
- Review (79) → In Progress (78)
- Review (79) → Done (80)
- Human Intervention (75) ↔ Human Answered (76)

## Project Status

### Completed Features
- User management with sessions
- Exercise library (CRUD)
- Workout planning with scheduling
- Live training with guided flow
- Dashboard with real stats
- PWA manifest and iOS fixes

### In Progress
- Training history with progression tracking

### Planned
- Exercise progression charts
- Body measurements tracking
- Rest timer between sets
- Personal records (PRs) detection
- Workout templates
- Progressive overload suggestions
- Full offline support

## Development Notes

- **docs_required:** `true` - Architecture, design-decision, and schema tickets must have linked documentation
- **Epics:** Special ticket types (not a status)
- **Human Intervention:** Use when blocking on human decision; move to `human_intervention` column and wait for `human_answered`
