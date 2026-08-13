# Camera App Implementation Plan

Based on the provided inspiration image and your requirements, here is a 5-phase implementation plan to upgrade the camera app's UI/UX and add the new features.

## Phase 1: Window Management & Base Layout
**Goal:** Remove the native OS window chrome and set up the fundamental dark-themed structure.
*   **Frameless Window:** Update `tauri.conf.json` to disable window decorations (`"decorations": false`) to remove the GNOME top bar.
*   **Custom Window Controls:** Implement a custom top-right close button that communicates with the Tauri backend to close the app.
*   **Base Layout:** Create the main vertical layout structure:
    *   Top status/toggle bar.
    *   Central camera viewport area.
    *   Bottom control panel.
*   **Theming:** Apply the global dark theme and premium styling (fonts, colors, spacing) matching the inspiration image.

## Phase 2: Camera Viewport & Overlays
**Goal:** Enhance the live camera feed area with requested visual features.
*   **Rounded Viewport:** Style the camera video element with prominent rounded corners and proper aspect ratio containment.
*   **Grid & Layout Guides:** Add a CSS overlay over the video feed for a "rule of thirds" grid. Include a toggle to show/hide the guides.
*   **Flash Mode (Photos Only):** Implement a full-screen or viewport-level white `div` overlay. When a photo is taken, this overlay will briefly transition to 100% opacity (bright white) and fade out, simulating a flash and temporarily illuminating the user's face.

## Phase 3: Top Bar Toggles & Indicators
**Goal:** Implement the top settings bar from the inspiration image.
*   **Aspect Ratio Toggle:** Add a clickable control to switch between different aspect ratios (e.g., 1:1, 4:3, 16:9). This should dynamically resize the camera viewport.
*   **File Format Toggle:** Add a toggle to switch the photo save format between **JPG** and **PNG**.
*   **Video FPS Toggle:** Add a toggle (visible in video mode) to switch between different frame rates (e.g., 30 FPS, 60 FPS).
*   **Icons:** Add the settings icon (left) and expand icon (right) to match the layout.

## Phase 4: Bottom Control Panel
**Goal:** Build the advanced control panel and main shutter area.
*   **Advanced Settings UI:** Create the custom slider/dial interfaces for advanced settings like Shutter Speed (SS) and Exposure Value (EV), matching the pill-shaped designs in the inspiration.
*   **Info Display:** Add the textual readouts for current settings (SS, EV, ISO) above the shutter button.
*   **Shutter Button:** Implement the large, prominent white circular shutter button.
*   **Secondary Controls:** Add the thumbnail preview (bottom left) and camera flip/switch button (bottom right).

## Phase 5: State Management & Polish
**Goal:** Connect the UI elements and refine the overall feel.
*   **State Wiring:** Ensure all the new toggles (Format, Aspect Ratio, FPS) update the central application state.
*   **Transitions & Animations:** Add smooth CSS transitions for hovering buttons, opening settings, and changing aspect ratios.
*   **Final Review:** Do a pixel-by-pixel comparison with the inspiration image to ensure the premium, sleek dark aesthetic is achieved. Test on Linux to confirm the GNOME top bar is gone and custom controls work flawlessly.
