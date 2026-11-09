//! **Administrator actions ask the person's password each time, through the
//! system's own window**
//! ([ADR 0150](../../../docs/decisions/0150-administrator-actions-ask-the-persons-password-each-time.md)).
//!
//! The owner, 2026-11-08 (ADR 0149): *"Ask my password each time"*; and on
//! 2026-11-09: *"Keep both in step"*, *"New and existing"*, *"Yes, alo's
//! desktop draws it"*. This crate is what three small privileged programs and
//! their callers share, and nothing here draws anything.
//!
//! # The three programs, and the one thing each may do
//!
//! | program | started by | may do |
//! |---|---|---|
//! | `alo-keeping-the-system-password` | a connection from the sign-in screen (root only) | write the person's password into the system's list, through `chpasswd` |
//! | `alo-checking-a-password` | a connection from anybody | run polkit's own helper **for the caller's own login**, and nothing else |
//! | `alo-running-as-administrator` | a connection from `alo-agentd` | ask polkit whether the person said yes, and only then run that one command as root |
//!
//! Each is started by systemd per connection (`Accept=yes`), so each holds its
//! privilege for one request and is gone. Who is on the other end is the
//! kernel's answer (`SO_PEERCRED`), never something the caller says.
//!
//! # What alo never does
//!
//! **alo never sees the password it is asked for.** The person types it into a
//! sheet the desktop draws (`alo-shell`), the desktop hands it to polkit's own
//! helper through [`the_helpers_conversation`], and polkit decides. The agent,
//! the model and the agent service see only *yes* or *no* — which is
//! [`answering_as_administrator::NotAsAdministrator`]'s whole vocabulary.
//!
//! # Map
//!
//! | | |
//! |---|---|
//! | `logins` | Who a number is, and whether they are an administrator |
//! | `keeping` | What the sign-in screen sends, and what the keeper answers |
//! | `in_step` | Whether the system's list holds the person's password, for the sheet's refusal |
//! | `the_helpers_conversation` | The desktop's side of polkit's helper |
//! | `answering_as_administrator` | The line `alo-agentd` and the runner speak |
//! | `what_polkit_said` | polkit's three answers, read as the person's |
//! | `running` | The runner's order: who asked, what, the person's yes, the run |
//! | `on_the_system_bus` | logind and polkit, asked on the system bus |

#![cfg(target_os = "linux")]

pub mod answering_as_administrator;
pub mod in_step;
pub mod keeping;
pub mod logins;
pub mod on_the_system_bus;
pub mod running;
pub mod the_helpers_conversation;
pub mod what_polkit_said;
