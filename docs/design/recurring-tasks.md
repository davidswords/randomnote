# Recurring tasks

How randomtask models tasks that repeat. Decided points are marked **Decided**. Everything else is open and listed at the end.

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

## When an occurrence is completed (Decided)

Like Reminders, a series repeats on a schedule. **The next occurrence is the first time the rule produces that is later than both the completed occurrence's due time and the moment it was completed.**

That one rule covers every case below. They come from how Reminders behaves, as shown in MacMost's "Using Repeating Reminders" video.

| Series | Completed | Next occurrence |
|---|---|---|
| Daily at 10:00 | Overdue, at 10:30 the same day | Tomorrow at 10:00 |
| Daily at 11:00 | Early, at 10:30 the same day | Tomorrow at 11:00 |
| Daily at 10:00 | Several days overdue, later in the day | Tomorrow at 10:00. Missed days are skipped. |
| Daily at 21:00 | Early, that afternoon | Tomorrow at 21:00 |
| Hourly, on the hour | Overdue | The start of the next hour |
| Every Monday | On Monday | Next Monday |
| Every Monday | Late, on Wednesday | Next Monday |
| Every Monday | Three weeks late | Next Monday. Missed weeks are skipped. |
| Every Monday | Early, on the Saturday before | The Monday after |
| Every 3 days, due Monday | On time | Thursday, then Sunday, Wednesday… |
| Every 3 days, due Monday | Thursday's done late, on Saturday | Sunday |

A late task never creates a backlog of overdue copies, and completing early never brings the same occurrence back.

**First occurrence.** If the chosen start date doesn't match the rule, the first occurrence is the first date on or after the start that does. For example, "first weekend day of the month" starting on a Wednesday 1 June gives Saturday 4 June.

**Not supported,** because Reminders doesn't offer it: repeating a fixed time after completion ("3 days after I last did it").

## Repeat options (Decided)

A repeating task always has a due date. Repeat is only available once a date is set.

- **Presets:** Never, Hourly, Daily, Weekdays, Weekends, Weekly, Biweekly, Monthly, Every 3 Months, Every 6 Months, Yearly.
- **Custom:**
  - **Daily:** every N days.
  - **Weekly:** every N weeks, on chosen weekdays.
  - **Monthly:** every N months, either on chosen days of the month (for example the 15th) or on an ordinal day: first, second, third, fourth, fifth or last, combined with a weekday, "weekday" or "weekend day" (for example the first Saturday, or the first weekend day).
  - **Yearly:** every N years, in chosen months, optionally on an ordinal day (for example the first Saturday of January and July).
- **Limits, as in Reminders:** hourly repeats run all 24 hours, with no time window. A series has one time of day; different times on different days need separate tasks.

## Time zones (Decided)

As in Reminders, times follow the time zone of the device's current location by default, so a task due at 09:00 is due at 09:00 local time after travelling. A setting can fix all tasks to one chosen time zone instead.

## To confirm on a Mac

The videos don't settle these. Each is checked in Reminders before it becomes a test.

- **Overdue, completed before the time of day.** A daily 10:00 task, several days overdue, completed today at 09:00. The rule above gives today at 10:00. Does Reminders give today or tomorrow?
- **Ending a repeat.** Which options exist, for example "never" and "on a date".
- **Editing.** Whether changing a repeating task applies to it and all future occurrences, with completed occurrences keeping what they had.
- **History.** Whether each completed occurrence shows as its own entry in the Completed list.
- **Ordinal names and limits.** The exact list of ordinals ("fifth", "last") and the range of N.
