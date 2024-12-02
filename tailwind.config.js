/** @type {import('../learning_webrust/tailwind-yew-setup-test2/tailwindcss').Config} */
module.exports = {
	content: ['./src/**/*.{html,rs}', 'index.html'],

	theme: {
		extend: {},
	},
	plugins: [
		require('@tailwindcss/typography'),
		require('@tailwindcss/forms'),
	],
};
