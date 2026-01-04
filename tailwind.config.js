/** @type {import('tailwindcss').Config} */
module.exports = {
  content: {
    files: [
      "*.html",
      "./app/**/*.rs",
      "./server/**/*.rs",
      "./frontend/**/*.rs",
    ],
  },
  safelist: [
    'btn',
    'btn-primary',
    'btn-secondary',
    'btn-accent',
    'card',
    'card-body',
    'card-title',
    'card-actions',
    'input',
    'input-bordered',
    'input-primary',
    'form-control',
    'label',
    'label-text',
    'tabs',
    'tabs-bordered',
    'tab',
    'tab-content',
  ],
  theme: {
    extend: {},
  },
  plugins: [require("daisyui")],
  daisyui: {
    themes: [
      {
        mytheme: {
          "primary": "#bef63b",
          "secondary": "#1e293b",
          "accent": "#60a5fa",
          "neutral": "#1f2937",
          "base-100": "#1e293b",
          "base-200": "#0f172a",
          "base-300": "#020617",
          "info": "#3b82f6",
          "success": "#10b981",
          "warning": "#f59e0b",
          "error": "#ef4444",
        },
      },
    ],
    darkTheme: "mytheme",
    base: true,
    styled: true,
    utils: true,
  },
}
