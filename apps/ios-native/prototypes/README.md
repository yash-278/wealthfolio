# Dashboard design prototype

Throwaway visual study: which dashboard hierarchy best fits the native app?

```sh
python3 -m http.server 8766 --bind 127.0.0.1 --directory apps/ios-native/prototypes
```

Open http://127.0.0.1:8766/dashboard-prototype.html?variant=A.
Use A/B/C or arrow keys to compare three layouts. Period selection, direct chart hover/drag and keyboard date
inspection, privacy, account previews and a Quick Add sheet use synthetic data.
No API calls, persistence, account changes or real balances are involved.

SwiftUI has no browser route, so this companion is kept beside the native source.
It evaluates hierarchy, density and chart composition. CSS materials are only a
visual approximation; Apple Liquid Glass must be reviewed in the native build.

A: spacious chart-led overview (recommended).
B: refined analytical overview, matching light and dark themes; Dashboard title,
borderless navigation and direct chart inspection. Use `&theme=light` or
`&theme=dark` to link to an appearance.
C: account-first overview with a compact chart.

Yash prefers B as the basis for further refinement; final approval is pending. After selection, implement the chosen design
in SwiftUI/Swift Charts and archive the prototype on a throwaway branch. Do not
ship these files as application UI.
