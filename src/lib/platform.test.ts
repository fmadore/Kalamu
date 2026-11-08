import { describe, expect, it } from 'vitest';
import { modifier, osNames, primary, shifted, shortcut, zoomsWithWheel } from './platform.js';

describe('platform', () => {
	it('takes Command on a Mac and Ctrl elsewhere as the shortcut modifier', () => {
		const command = { metaKey: true, ctrlKey: false } as KeyboardEvent;
		const ctrl = { metaKey: false, ctrlKey: true } as KeyboardEvent;
		expect(primary(command, true)).toBe(true);
		expect(primary(ctrl, true)).toBe(false);
		expect(primary(ctrl, false)).toBe(true);
		expect(primary(command, false)).toBe(false);
	});

	it('writes a shortcut as each system does', () => {
		expect(shortcut('N', true)).toBe('⌘N');
		expect(shortcut('N', false)).toBe('Ctrl N');
		expect(shortcut('↵', true)).toBe('⌘↵');
		expect(shifted('F', true)).toBe('⇧F');
		expect(shifted('F', false)).toBe('Shift F');
		expect(modifier(true)).toBe('⌘');
		expect(modifier(false)).toBe('Ctrl');
	});

	it('zooms with Ctrl and the wheel everywhere, and with Command too on a Mac', () => {
		const command = { metaKey: true, ctrlKey: false } as WheelEvent;
		const ctrl = { metaKey: false, ctrlKey: true } as WheelEvent;
		expect(zoomsWithWheel(ctrl, false)).toBe(true);
		expect(zoomsWithWheel(ctrl, true)).toBe(true);
		expect(zoomsWithWheel(command, true)).toBe(true);
		expect(zoomsWithWheel(command, false)).toBe(false);
	});

	it('names the keychain and the tray for each system', () => {
		expect(osNames(false).keychain).toContain('Credential Manager');
		expect(osNames(true).keychain).toContain('Keychain');
		expect(osNames(true).tray).toContain('menu bar');
	});
});
