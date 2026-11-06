// @ts-check
// Not a style linter: Prettier owns formatting and svelte-check owns types. These are the checks
// those two cannot make, as in Live Translation: promises nobody handles and Svelte mistakes.
import { defineConfig } from 'eslint/config';
import svelte from 'eslint-plugin-svelte';
import ts from 'typescript-eslint';

export default defineConfig(
	{
		ignores: [
			'build/',
			'.svelte-kit/',
			'node_modules/',
			'src-tauri/',
			'crates/',
			'target/',
			'docs/',
			'.claude/'
		]
	},
	{
		files: ['**/*.{js,mjs,ts}'],
		languageOptions: { parser: ts.parser }
	},
	svelte.configs.base,
	{
		files: ['**/*.svelte', '**/*.svelte.ts'],
		languageOptions: { parserOptions: { parser: ts.parser } }
	},
	{
		languageOptions: {
			parserOptions: {
				projectService: {
					allowDefaultProject: [
						'eslint.config.js',
						'scripts/*.mjs',
						'playwright.config.ts',
						'e2e/*.ts'
					],
					defaultProject: 'tsconfig.json',
					// The files above, outside tsconfig.json, are linted with a default project:
					// a handful of small scripts, above the parser's default of eight.
					maximumDefaultProjectFileMatchCount_THIS_WILL_SLOW_DOWN_LINTING: 16
				},
				tsconfigRootDir: import.meta.dirname,
				extraFileExtensions: ['.svelte']
			}
		},
		plugins: { '@typescript-eslint': ts.plugin },
		rules: {
			'@typescript-eslint/no-floating-promises': 'error',
			'@typescript-eslint/no-misused-promises': [
				'error',
				{ checksVoidReturn: { attributes: false } }
			],
			'@typescript-eslint/switch-exhaustiveness-check': 'error',
			'svelte/no-at-html-tags': 'error',
			'svelte/valid-compile': 'error',
			'svelte/require-each-key': 'error',
			'svelte/no-unused-svelte-ignore': 'error'
		}
	}
);
