# Current · Off-peak reminder

A household wants to run a flexible appliance when energy is cheaper. From Home energy, the user sets a recurring reminder, checks the schedule, and sees when the next reminder will arrive.

[Play the prototype](https://rust-js-lang.github.io/agent-pixel/?project=current-reminders&screen=overview&preview) · [Inspect the design layers](https://rust-js-lang.github.io/agent-pixel/?project=current-reminders)

## Screen map

~~~text
Home energy --> Set schedule --> Review --> Saving --> Confirmation
                    ^               ^         |
                    |               |         v
                    +---- Back -----+     Save failure
                                    +-------- Retry
~~~

The prototype walks through the selected weekday example. Use **Simulate save failure** on the review screen to inspect the retry branch.

| Screen | User action | Expected result |
| --- | --- | --- |
| [Home energy](https://rust-js-lang.github.io/agent-pixel/?project=current-reminders&screen=overview) | Select **Set off-peak reminder** | Open setup with the next off-peak start time and weekdays as the defaults. |
| [Set schedule](https://rust-js-lang.github.io/agent-pixel/?project=current-reminders&screen=schedule) | Choose a time and toggle repeat days, then select **Review reminder** | Show the chosen schedule. Require a valid local time and at least one day. Keep selections if the user goes back. |
| [Review](https://rust-js-lang.github.io/agent-pixel/?project=current-reminders&screen=review) | Select **Save reminder** | Ask for push permission if needed. When permission is granted, save the reviewed schedule. While saving, disable the action and show **Saving reminder…**. |
| [Confirmation](https://rust-js-lang.github.io/agent-pixel/?project=current-reminders&screen=success) | Check the next occurrence and select **Back to Home energy** | Show the next firing time from the saved schedule. Home energy now offers **Manage reminder** with edit and turn-off actions. |
| [Save failure](https://rust-js-lang.github.io/agent-pixel/?project=current-reminders&screen=error) | Select **Try again** | Retain the pending draft, return to Review, and let the user retry the same save. |

## Behavior and data

- Store the selected local time as minutes after midnight, repeat days as ISO weekdays from 1 through 7, and the IANA time zone used at confirmation. Render times using the user’s locale. A schedule is recurring at the selected wall-clock time in its saved zone.
- Determine the next occurrence by advancing to the first selected weekday and time strictly after now. When clocks skip the selected wall time, fire at the first valid time afterward. When a time occurs twice, fire once at its earlier occurrence.
- When permission is denied, stay on Review and show **Notifications are off** with a way to open notification settings. Keep the draft. Confirmation follows permission and a successful save.
- Keep the draft available after a network error. Reusing the same operation ID on retry must produce a single reminder. When editing a saved reminder, leave the previous schedule active until the update succeeds.

## Layout and accessibility

The artboards are 480 × 600 reference views. In production, use a single-column layout with 24 px page gutters and a 480 px maximum content width. Let text wrap and content scroll on shorter or narrower screens. The primary action is 50 px high in the reference. Weekday targets should have at least a 44 × 44 px hit area even when the visual chip is smaller.

| Role | Reference |
| --- | --- |
| Page surface | `#F2F4ED` |
| Card surface | `#FFFFFF` |
| Primary action | `#133D35` with white text |
| Main text | `#162D29` |
| Secondary text | `#53645C` |
| Failure surface and heading | `#F7E8E2` and `#8C392A` |

Use a labeled time control and toggle buttons with full weekday names and selected state. Keep focus visible. On screen change, move focus to the new heading. Announce saving and save failure through an appropriate live region. Use text labels alongside colors for selection and errors.

## Acceptance criteria

- A user can choose 9:00 PM Monday through Friday, review it, save once, and see the correct next reminder in their saved time zone.
- Back from Review preserves the time and days. Clearing all days explains what to select and prevents continuing.
- A denied permission shows recovery on Review and produces no reminder. A repeated submit while a save is pending creates no duplicate.
- An interrupted save preserves the draft and offers a retry. If the original request succeeded despite the interruption, retry resolves to that reminder.
- The setup controls, status messages, and primary actions stay usable at 320 px width and with keyboard navigation.
