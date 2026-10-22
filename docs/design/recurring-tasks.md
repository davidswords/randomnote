# Recurring tasks

How randomnote models tasks that repeat. Decided points are marked **Decided**. Everything else is open and listed at the end.

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

## Two ways to repeat (Decided)

A series' rule says which date the next occurrence counts from.

### Schedule-based: follows the calendar

The next due date comes from the rule and the previous due date, whenever the task was actually done.

*Example: "Put the bins out, every Monday."*

- Completed on Monday → the next one is due next Monday.
- Completed late, on Wednesday → the next one is still due next Monday. Bin day doesn't move.

### Completion-based: counts from when it was done

The next due date is a fixed interval after the day the task was completed.

*Example: "Water the plants, 3 days after I last did it."*

- Completed on Monday → the next one is due Thursday.
- Completed late, on Saturday → the next one is due Tuesday, 3 days after it was actually done.

### Intervals work in both modes

"Every 3 days" can be either kind. Apple Reminders' "every 3 days" is schedule-based: it counts from the previous due date. For a task due Monday that repeats every 3 days:

| | Schedule-based | Completion-based |
|---|---|---|
| Due dates when done on time | Mon, Thu, Sun, Wed… | Mon, Thu, Sun, Wed… |
| Thursday's one done late, on Saturday | Next is due Sunday (Thursday + 3) | Next is due Tuesday (Saturday + 3) |

The two modes only differ when a task is done late.

## Open questions

- **Very late completion of a schedule-based task.** If a weekly task is completed three weeks late, is the next one due on the next upcoming scheduled date, so missed weeks are skipped, or on the scheduled date after the one just completed, even if that is already in the past?
- **Editing.** Does changing a recurring task apply to this occurrence only, or to this and all future occurrences?
- **Due dates on ordinary tasks.** Recurrence needs due dates. Can one-off tasks have due dates too?
- **Which rules are supported.** For example every N days, weeks, months or years; particular weekdays; the last day of the month.
- **Ending a series.** After a number of occurrences, on a date, or only by deleting it?
- **Time zones.** Whether "due Monday" is a calendar date or a moment in time, and what happens when travelling.
