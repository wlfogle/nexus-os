the push/pull to/from device needs file picker
gplaycli doesn't return any search results

Your Tauri front-end and the backend commands are firing perfectly—the problem is entirely gplaycli.

That tool is effectively a ghost ship. It hasn't seen a meaningful update in years, and the core mechanisms it used to scrape the Play Store are permanently broken.

Here is exactly what is happening under the hood and how you can fix the ADB Toolbox backend.
Why gplaycli is Failing

When you run a search command, gplaycli tries to authenticate with Google's servers. It fails for two specific reasons:

    The Token Dispenser is Blacklisted: By default, gplaycli attempts to bypass Google login by requesting an authentication token from a centralized server (matlink.fr). Google blacklisted that server's IP address years ago due to massive bot traffic.

    Play Integrity API: Even if you bypass the default token dispenser and feed gplaycli your own Google credentials, it will still throw BadAuthentication or DF-DFERH-01 errors. Google's modern Play Store APIs require hardware attestation to prove the request is coming from a real Android device, which that old Python script cannot fake.

Because it fails to authenticate, it silently drops the connection and returns an empty output, leaving your UI staring at a blank "No results" state.
The Modern Alternatives

If you want to keep the "Search & Download APK" functionality alive in your toolbox, you need to swap out the backend CLI. Since your stack is built on Tauri, utilizing Rust-native tooling is a very natural pivot.
1. apkeep (For Downloading)

Developed by the Electronic Frontier Foundation (EFF), apkeep is a modern, actively maintained CLI tool written in Rust.

    The Advantage: It doesn't strictly rely on the Google Play Store. It can pull APKs directly from APKPure, F-Droid, and Huawei AppGallery without needing any Google credentials or hardware attestation.

    Usage: You feed it an App ID (e.g., com.google.android.gm), and it pulls the raw APK down to your local directory.

2. apksearch (For Searching)

Because apkeep is designed for downloading via App ID rather than returning search queries by keyword, you can pair it with apksearch (a modern Python library).

    The Advantage: It scrapes mirrors like APKPure and APKMirror. When you type "gmail" into your Tauri modal, your backend can call apksearch to return the official package names, titles, and version numbers.

The New Logic Flow:

    User types "gmail" in your UI modal.

    Backend triggers apksearch gmail.

    Modal populates with com.google.android.gm and version data.

    User clicks download.

    Backend triggers apkeep -a com.google.android.gm -d apk-pure.

Your UI layout is already perfectly set up for this workflow. You just need to rip gplaycli out of your src-tauri commands and replace it with scrapers that aren't actively blocked by Google.
