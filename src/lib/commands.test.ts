// The commands the interface calls, held to the shell's. Rust's tests tie the shell's handler and
// its capability to `COMMANDS` (`src-tauri/src/lib.rs`); this ties the bridge to them: every
// name it calls exists, every command is called, and each call passes the arguments the
// command takes, by the names Tauri gives them (camelCase). The development mock answers only
// commands that exist.

import { readdirSync, readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import { DESKTOP_ONLY, NO_RESULT } from './dev/noResult.js';

/** The files that call the shell's commands: the bridge, and the part kept beside it. */
const BRIDGE = ['src/lib/tauri.ts', 'src/lib/tauriCited.ts'];

/** `COMMANDS` in `command_names.rs`, in its order. */
function shellCommands(): string[] {
	const source = readFileSync('src-tauri/src/command_names.rs', 'utf8');
	const list = source.slice(source.indexOf('pub const COMMANDS'));
	return [...list.slice(0, list.indexOf('];')).matchAll(/"([a-z_]+)"/g)].map((m) => m[1]);
}

/** Each command's arguments as the interface passes them, by the shell's function: the
 * parameters that are not Tauri's own (state, the app, the window), camelCased. An `Option`
 * may be left out. */
function shellArguments(): Map<string, { name: string; optional: boolean }[]> {
	const dir = 'src-tauri/src/commands';
	const found = new Map<string, { name: string; optional: boolean }[]>();
	for (const file of readdirSync(dir).filter((f) => f.endsWith('.rs'))) {
		const source = readFileSync(`${dir}/${file}`, 'utf8');
		for (const m of source.matchAll(
			/#\[tauri::command\][\s\S]*?\bfn\s+([a-z_0-9]+)\s*(?:<[^>]*>)?\(/g
		)) {
			// The parameters, to the parenthesis that closes them.
			let depth = 1;
			let end = m.index + m[0].length;
			while (depth > 0 && end < source.length) {
				const c = source[end++];
				if (c === '(' || c === '<' || c === '[') depth++;
				else if (c === ')' || c === '>' || c === ']') depth--;
			}
			const params = source.slice(m.index + m[0].length, end - 1);
			const parts: string[] = [];
			let level = 0;
			let current = '';
			for (const c of params) {
				if (c === '<' || c === '(' || c === '[') level++;
				if (c === '>' || c === ')' || c === ']') level--;
				if (c === ',' && level === 0) {
					parts.push(current);
					current = '';
				} else current += c;
			}
			parts.push(current);
			found.set(
				m[1],
				parts
					.map((p) => p.trim())
					.filter((p) => p.includes(':'))
					.map((p) => {
						const at = p.indexOf(':');
						return {
							name: p
								.slice(0, at)
								.trim()
								.replace(/^mut\s+/, ''),
							type: p.slice(at + 1).trim()
						};
					})
					.filter(({ type }) => !/^(tauri::)?(State|AppHandle|Window|WebviewWindow)\b/.test(type))
					.map(({ name, type }) => ({
						name: name.replace(/_([a-z0-9])/g, (_, c: string) => c.toUpperCase()),
						optional: type.startsWith('Option<')
					}))
			);
		}
	}
	return found;
}

/** Each call in the bridge: the command's name and the arguments' names it passes. */
function bridgeCalls(): { command: string; args: string[] }[] {
	return BRIDGE.flatMap((file) =>
		[
			...readFileSync(file, 'utf8').matchAll(
				/invoke(?:<[^>]*>)?\(\s*'([a-z_]+)'\s*(?:,\s*\{([^}]*)\})?\s*\)/g
			)
		].map((m) => ({
			command: m[1],
			args: (m[2] ?? '')
				.split(',')
				.map((a) => a.split(':')[0].trim())
				.filter(Boolean)
		}))
	);
}

describe('the commands the interface calls', () => {
	const commands = shellCommands();
	const calls = bridgeCalls();

	it('reads the shell’s list and the bridge', () => {
		expect(commands.length).toBeGreaterThan(150);
		expect(commands).toContain('log_interface_error');
		expect(calls.length).toBeGreaterThan(150);
		// Tauri's own parameters left out, the others camelCased.
		expect(shellArguments().get('units_view')).toEqual(
			['jobId', 'setupId', 'sourceId'].map((name) => ({ name, optional: false }))
		);
		expect(calls.find((c) => c.command === 'units_view')?.args).toEqual([
			'jobId',
			'setupId',
			'sourceId'
		]);
	});

	it('are commands the shell has', () => {
		const unknown = calls.map((c) => c.command).filter((c) => !commands.includes(c));
		expect(unknown).toEqual([]);
	});

	it('call every command the shell has', () => {
		const called = new Set(calls.map((c) => c.command));
		expect(commands.filter((c) => !called.has(c))).toEqual([]);
	});

	it('pass the arguments each command takes, by their names', () => {
		const taken = shellArguments();
		const wrong = calls.flatMap(({ command, args }) => {
			const params = taken.get(command);
			if (!params) return [`${command}: no #[tauri::command] function`];
			const missing = params.filter((p) => !p.optional && !args.includes(p.name));
			const extra = args.filter((a) => !params.some((p) => p.name === a));
			return [
				...missing.map((p) => `${command}: ${p.name} not passed`),
				...extra.map((a) => `${command}: ${a} not taken`)
			];
		});
		expect(wrong).toEqual([]);
	});

	it('are the only ones the development mock answers', () => {
		const mocks = readdirSync('src/lib/dev')
			.filter((f) => f.endsWith('.ts') && !f.endsWith('.test.ts'))
			.map((f) => readFileSync(`src/lib/dev/${f}`, 'utf8'))
			.join('\n');
		const answered = [...new Set([...mocks.matchAll(/case '([a-z]+_[a-z_]+)'/g)].map((m) => m[1]))];
		const listed = [...NO_RESULT, ...DESKTOP_ONLY];
		expect([...answered, ...listed].filter((c) => !commands.includes(c))).toEqual([]);
	});
});
