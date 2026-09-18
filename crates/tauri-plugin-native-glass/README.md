# Native iOS controls

This local Tauri plugin renders SwiftUI controls over the iOS web view using
Apple Liquid Glass on iOS 26 and later. The React app supplies layout rectangles,
translated labels, selection state and action IDs. The plugin emits action IDs;
React continues to own routing, portfolio data and server synchronization.

Covered controls are the mobile navigation bar, its More menu, dashboard view
selection, balance privacy and dashboard actions. The bottom bar exposes Dashboard,
Activities, Insights and Assistant; More retains Quick Add, Search and other routes.
The mobile top controls are fixed outside the scroll container. Native selection
animates immediately on touch, then reconciles with the routed selection. Reduce
Motion disables that animation. Portfolio content and forms
remain web views. Web, desktop, Android and older iOS versions retain their
existing controls.

`NativeGlassControls` reserves the existing layout, hides its web controls only
after the native plugin acknowledges support, and removes the native host on
unmount. Hosts occupy only the control rectangle. Controls hide while a web
modal or text input is active. Theme changes, resizing and scrolling update the
native host. SwiftUI supplies accessibility labels and selected traits.

Build using the personal iOS configuration and the existing Xcode signing setup.
Generated `.tauri` dependencies are build output and must not be committed.
Physical installation does not establish visual or interaction acceptance.
