# User Journeys

## First-time setup (PC)

1. Download the installer from GitHub Releases, run it.
2. The installer: copies `mikey`, registers the virtual camera (one admin prompt on Windows), adds a firewall rule for private networks, enables autostart, launches the tray app.
3. The tray icon appears. If the virtual mic driver is missing, the tray icon turns amber and the flyout displays an actionable card: *"One more step: install the virtual microphone"* → clicking it opens the download page. Mikey detects it automatically once installed; no restart of Mikey needed.
4. Done. The user never opens the PC side again unless they want to.

## First-time setup (phone)

1. Install the APK (GitHub release / F-Droid).
2. Open Mikey. The screen shows the dimmed camera (top) and dimmed mic (bottom). No onboarding carousel.
3. Mikey is already searching for the PC. The status dot turns green when found.
4. Tapping the mic the first time asks for microphone permission (and notification permission on Android 13+). Tapping the camera the first time asks for camera permission. Nothing is asked up front.

## The first connection on each level (one-time friction)

| Level | One-time step |
|---|---|
| 1 USB debugging | Phone shows *"Allow USB debugging?"* → tick *Always allow from this computer* → Allow. The PC tray icon turns amber and the flyout/notification says *"Tap Allow on your phone"* while waiting. |
| 2 USB tethering | User turns on USB tethering (Mikey offers a shortcut to that settings page when it sees a USB cable but no Level 1/2 link). |
| 3 Bluetooth | Phone and PC must be paired once in the OS Bluetooth settings. Mikey asks for the Bluetooth permission the first time it tries this level. |
| 4 Wi‑Fi | First time only: the PC shows *"Pixel 7 wants to connect — Allow / Deny"* via notification and flyout banner. After that, it joins silently. |

## Daily use

1. Open Mikey on the phone (or plug in via USB with debugging on — the PC can open the app for you, [pc-ux.md](pc-ux.md)).
2. Tap mic. It's live. The status dot is green; the notification says *"Mic on · USB"*.
3. Minimize, lock the screen, take the call.
4. When done, swipe Mikey away from Recents, or tap *Stop* in the notification. Everything disconnects and the PC's virtual devices go silent/black.

## Mid-call upgrade

On Wi‑Fi, battery dropping — the user plugs in the USB cable. Mikey detects the better link, opens it, moves the stream over, and closes Wi‑Fi. The other person hears at most a brief blip (target < 300 ms). The drawer and notification now say *USB*.
