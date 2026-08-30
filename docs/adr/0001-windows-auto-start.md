# ADR 0001: Windows Auto-Start on Login

## Status
Accepted

## Context

TustPortal is a background auto-login tool. Its entire value proposition depends on the application running when the user connects to the campus network. The current macOS version does not register itself to start on user login.

During the Windows port, we identified that Windows users restart their machines more frequently than macOS users (system updates, shutdowns, power cycles). If the application is not running after login, the user will hit the captive portal manually and the tool fails at its primary job.

The decision is hard to reverse because once users expect the app to auto-start, removing the feature would degrade daily UX. It is also surprising without context why a background utility would *not* auto-start. Finally, there are genuine alternatives (no auto-start, system service, scheduled task, registry run key), each with different complexity and permission trade-offs.

## Decision

We will implement **Auto-Start on login for Windows only**, with the following constraints:

1. **Opt-in by prompt:** On the first application launch (when no credentials exist), a native dialog will ask the user whether to enable auto-start. The default will be "Yes."
2. **Mutable in Settings:** A toggle will be added to the Settings page so users can change their preference later.
3. **Windows-only:** The feature will be gated behind `#[cfg(target_os = "windows")]` in Rust. The macOS version will not be modified.
4. **Frontend awareness:** The Vue frontend will conditionally render the Auto-Start toggle only when running on Windows, via a platform-detection command.

We will implement this using the Windows Registry (`HKCU\Software\Microsoft\Windows\CurrentVersion\Run`) rather than a system service, because it requires no elevation and matches the per-user scope of the application.

## Consequences

- **Positive:** The Windows port behaves like a true background utility. Users do not need to manually launch it after every reboot.
- **Positive:** The opt-in prompt respects user agency and avoids surprising behavior.
- **Negative:** The frontend must now handle platform-conditional UI rendering, adding a small cross-layer dependency (Rust → Vue platform signal).
- **Negative:** The macOS version retains the gap of not auto-starting, but this is an explicit constraint to avoid scope creep.
- **Risk:** If the registry write fails (e.g., restricted corporate Windows image), the app must degrade gracefully and log the failure without crashing.
