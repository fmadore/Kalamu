import { describe, expect, it } from 'vitest';
import {
	alted,
	backShortcut,
	isBack,
	modifier,
	osNames,
	primary,
	shifted,
	shortcut,
	zoomsWithWheel
} from './platform.js';

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

	it('goes back with Alt and the left arrow, and with ⌘[ on a Mac only', () => {
		const key = (key: string, held: Partial<KeyboardEvent> = {}) =>
			({
				key,
				altKey: false,
				ctrlKey: false,
				metaKey: false,
				shiftKey: false,
				...held
			}) as KeyboardEvent;
		expect(isBack(key('ArrowLeft', { altKey: true }), false)).toBe(true);
		expect(isBack(key('ArrowLeft', { altKey: true }), true)).toBe(true);
		expect(isBack(key('[', { metaKey: true }), true)).toBe(true);
		expect(isBack(key('[', { metaKey: true }), false)).toBe(false);
		expect(isBack(key('ArrowLeft'), false)).toBe(false);
		expect(isBack(key('ArrowLeft', { altKey: true, shiftKey: true }), false)).toBe(false);
		expect(isBack(key('ArrowLeft', { altKey: true, ctrlKey: true }), false)).toBe(false);
	});

	it('writes Back for each system, with no key in a review on Windows', () => {
		expect(alted('←', true)).toBe('⌥←');
		expect(alted('←', false)).toBe('Alt ←');
		expect(backShortcut(false, false)).toBe('Alt ←');
		expect(backShortcut(true, false)).toBeNull();
		expect(backShortcut(true, true)).toBe('⌘[');
	});
});
