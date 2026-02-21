/** @type {import('tailwindcss').Config} */
export default {
  darkMode: "class",
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        primary: "#FF3B2E",
        background: {
          dark: "#0A0A0A",
          elevated: "#141414",
          card: "#1A1A1A",
        },
        border: {
          subtle: "rgba(255, 255, 255, 0.05)",
          medium: "rgba(255, 255, 255, 0.1)",
        },
        text: {
          primary: "#FFFFFF",
          secondary: "rgba(255, 255, 255, 0.7)",
          muted: "rgba(255, 255, 255, 0.5)",
        },
        glow: {
          red: "rgba(255, 59, 46, 0.3)",
        },
      },
      spacing: {
        section: "120px",
        "section-sm": "80px",
      },
      borderRadius: {
        card: "20px",
      },
      fontSize: {
        "display-xl": ["4.5rem", { lineHeight: "1", letterSpacing: "-0.02em", fontWeight: "700" }],
        "display-lg": ["4rem", { lineHeight: "1.05", letterSpacing: "-0.02em", fontWeight: "700" }],
        "heading-xl": ["3rem", { lineHeight: "1.1", letterSpacing: "-0.01em", fontWeight: "700" }],
      },
      boxShadow: {
        "glow-red": "0 0 60px rgba(255, 59, 46, 0.3)",
        "card-hover": "0 8px 32px rgba(0, 0, 0, 0.4)",
      },
    },
  },
  plugins: [],
}
