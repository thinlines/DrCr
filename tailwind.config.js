/** @type {import('tailwindcss').Config} */

import tailwindcssforms from '@tailwindcss/forms';

export default {
	content: [
		"./index.html",
		"./src/**/*.{vue,js,ts,jsx,tsx}",
	],
	theme: {
		extend: {},
		fontFamily: {
			"sans": ["Roboto Flex", "Helvetica", "Arial", "sans-serif"],
		}
	},
	plugins: [
		tailwindcssforms,
	],
}
