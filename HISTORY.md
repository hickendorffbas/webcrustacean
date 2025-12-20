0.7.0 [UNRELEASED]
===================
- Very basic flex layout support
- New improved CSS parser
- Horizontal scrolling is now supported


0.6.0
===================
- Improved content selection logic
- Basic cookie support
- If a url with protocol is entered, https and http are tried
- New, more correct javascript parser added
- Form improvements: hidden values are sent, and GET forms are supported


0.5.0
===================
- The main window is now resizable
- Added basic html tables
- More whitespace correctness (less whitespace shown in general)
- Bold and Italic text styles supported
- More image types supported in the rendering


0.4.0
===================
- Scrollbar bugfixes
- Added textfields and submit buttons for forms
- Added the ability to do POST requests from forms
- Button clicks are registered more reliably


0.3.0
===================
- No longer depend on SDL for font rendering
- Move to Rust 2021
- Implement very basic javascript interpretation
- Implement javascript functions
- Text selection, copy and paste support in the addressbar, and we select all on first click


0.2.0
===================
- Added ability to select and copy text
- Allow pasting in the url bar
- Images are now loaded in parallel
- Layout can now relayout parts of the layout, depending on changes in the DOM
- Repeating keys now work
- The main page load now happens in a seperate thread
- Added about:home


0.1.0
===================
- basic page loading and display
- basic css color parsing
- basic image loading
- basic history navigation
- renamed to webcrustacean
