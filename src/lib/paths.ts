// File paths as the app hands them over, written with either system's separator.

/** A path's folders and file name, without empty parts. */
export function pathParts(path: string): string[] {
	return path.split(/[\\/]/).filter(Boolean);
}

/** The last part of a path: a file's name, or a folder's; empty for an empty path. */
export function baseName(path: string): string {
	return pathParts(path).pop() ?? '';
}
