# Fenics Lang Website

A modern, responsive website for the Fenics programming language.

## Structure

```
website/
├── index.html              # Main HTML file
├── styles/
│   ├── main.css           # Core styles and layout
│   ├── components.css     # Component-specific styles
│   └── syntax.css         # Syntax highlighting for code
├── js/
│   ├── navigation.js      # Navigation and scrolling logic
│   ├── tabs.js           # Tab switching functionality
│   └── playground.js     # Interactive code playground
└── README.md             # This file
```

## Features

### 🎨 Modern Design

- Clean, minimalist interface
- Gradient accents and smooth animations
- Dark theme optimized for developers
- Fully responsive (mobile, tablet, desktop)

### 📱 Responsive Layout

- Mobile-first approach
- Hamburger menu for mobile devices
- Flexible grid system
- Touch-friendly interactions

### 🚀 Interactive Playground

- Live code editor with syntax highlighting
- Mock interpreter for demos
- Auto-save functionality (localStorage)
- Keyboard shortcuts (Ctrl/Cmd + Enter to run)

### 📚 Documentation

- Tabbed code examples
- Syntax highlighting for Fenics code
- Feature showcase
- Quick start guide

## Modules

### CSS Modules

**main.css**

- CSS variables for theming
- Base typography and layout
- Section styles
- Responsive breakpoints
- Animations and transitions

**components.css**

- Button styles
- Card components
- Code windows
- Tabs interface
- Playground layout

**syntax.css**

- Fenics language syntax highlighting
- Color scheme for keywords, strings, etc.
- Code block styling

### JavaScript Modules

**navigation.js**

- Hamburger menu toggle
- Smooth scrolling
- Active section highlighting
- Sticky navbar behavior

**tabs.js**

- Tab switching functionality
- Active state management

**playground.js**

- Code editor functionality
- Mock interpreter
- Run/clear actions
- Auto-save feature
- Keyboard shortcuts

## Customization

### Changing Colors

Edit CSS variables in `styles/main.css`:

```css
:root {
  --primary-color: #6366f1;
  --secondary-color: #8b5cf6;
  --accent-color: #ec4899;
  /* ... more variables */
}
```

### Adding New Sections

1. Add section HTML in `index.html`
2. Add section link to navigation
3. Style in appropriate CSS file
4. Update `navigation.js` if needed

### Modifying Code Examples

Edit the tab content in the `#examples` section of `index.html`.

## Browser Support

- Chrome (latest)
- Firefox (latest)
- Safari (latest)
- Edge (latest)
- Mobile browsers

## Development

To run locally:

1. Open `index.html` in a web browser
2. Or use a local server:

   ```bash
   # Python
   python -m http.server 8000

   # Node.js
   npx serve
   ```

3. Navigate to `http://localhost:8000`

## Future Enhancements

- [ ] Connect to actual Fenics interpreter backend
- [ ] Add more code examples
- [ ] Blog/news section
- [ ] Package manager documentation
- [ ] API reference
- [ ] Video tutorials
- [ ] Community showcase

## License

Same as Fenics Lang project.
