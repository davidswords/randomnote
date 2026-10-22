# Recurring tasks

How randomnote models tasks that repeat. Decided points are marked **Decided**. Everything else is open and listed at the end.

## Guiding principle (Decided)

Recurring tasks behave **exactly like Apple Reminders**. Where this note and Reminders disagree, Reminders wins. Reminders' behaviour is confirmed on a Mac before it becomes a test case.

## Model: series plus occurrences (Decided)

A recurring task is made of two kinds of record:

- A **series** holds what repeats: the title, the tags and the recurrence rule. It has its own ID.
- An **occurrence** is an ordinary task that belongs to a series and is due on one date.

Only the **next occurrence** exists as an open task. When it is completed, it stays as a completed record, which is the history of the series, and the next occurrence is created. Nothing is generated ahead of time, so a daily task never fills the database with future copies.

### Why not the alternatives

- **One task that moves forward, plus a completion log.** Simpler, but past completions would be log entries rather than tasks, so one past occurrence couldn't have its own tags or edits.
- **Occurrences computed on the fly from the rule,** as calendars do. Powerful, but complex, and fits a todo app poorly.

## Occurrence IDs and sync

Two devices can complete the same occurrence while offline. If each one gave the next occurrence a new random ID, syncing would produce two copies of it.

So an occurrence's ID is **calculated** from its series ID and its due date, not generated randomly. Both devices calculate the same ID for the next occurrence, and sync merges them into one task.

- **Ordinary tasks** get a random unique ID, created on the device.
- **Series** get a random unique ID, created on the device.
- **Occurrences** get an ID calculated from the series ID and the due date.

Everywhere else, an ID is an opaque string.

## Repeats follow the calendar (Decided)

Like Reminders, a series repeats on a schedule. The next due date comes from the rule and the previous due date, not from when the task was actually done.

*Example: "Put the bins out, every Monday."*

- Completed on Monday → the next one is due next Monday.
- Completed late, on Wednesday → the next one is still due next Monday. Bin day doesn't move.
- Completed three weeks late → the next one is due next Monday. Missed occurrences are skipped, so a late task never creates a backlog of overdue copies.
- Completed early, on the Saturday before → the next one is due the Monday after.

So the next due date is the first scheduled date after whichever is later: the day the task was completed, or the completed occurrence's due date. The late and early cases are expected to match Reminders, and are confirmed on a Mac before becoming tests.

Intervals work the same way. A task due Monday that repeats every 3 days is due Monday, Thursday, Sunday and so on. If Thursday's one is done late, on Saturday, the next one is still due Sunday.

**Not supported:** repeating a fixed time after completion ("3 days after I last did it"), because Reminders doesn't offer it.

## Following Reminders (to confirm on a Mac)

These defaults come from how Reminders is understood to work. Each needs confirming before it becomes a test.

- **Rules:** daily, weekly, every 2 weeks, monthly, every 3 months, every 6 months, yearly. Custom: every N days; every N weeks on chosen weekdays; every N months on chosen days of the month or on "the first Monday"-style days; every N years.
- **Ending a series:** never, or on an end date.
- **Editing:** changing the open occurrence changes the series, so it applies to it and all future occurrences. Completed occurrences keep what they had.
- **Due dates:** any task can have a due date, with or without a time, whether it repeats or not.

## Open questions

- **Time zones.** Whether "due Monday" is a calendar date or a moment in time, and what happens when travelling.
