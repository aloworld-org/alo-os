// Firefox on alo OS starts with sharing data with Mozilla switched off.
//
// The owner, 2026-11-13, asked whether Firefox's own default of sending
// diagnostic and interaction data to Mozilla should stand on a machine whose
// first law is "nothing leaves silently": "Off by default".
//
// Every line is `pref`, which is a default. None is `lockPref`, so each box
// stays in Firefox's own Settings, under Privacy & Security, unticked, and a
// person who wants to share can tick it (CLAUDE.md, law 5). An enterprise
// policy would lock them and tell the person their browser is managed by an
// organisation, which on a personal machine is not true.
//
// Firefox from Flathub reads this directory as /app/etc/firefox/defaults/pref,
// through its org.mozilla.firefox.systemconfig extension
// (docs/quirks/flatpak-firefox-reads-system-settings-from-an-extension.md).
// crates/alo-image holds every line of this file.

// "Allow Firefox to send technical and interaction data to Mozilla"
pref("datareporting.healthreport.uploadEnabled", false);
// "Send daily usage ping to Mozilla"
pref("datareporting.usage.uploadEnabled", false);
// "Allow Firefox to install and run studies"
pref("app.shield.optoutstudies.enabled", false);
// "Allow Firefox to send backlogged crash reports on your behalf"
pref("browser.crashReports.unsubmittedCheck.autoSubmit2", false);
