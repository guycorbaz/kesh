// Re-export from utils/cn.ts for shadcn-svelte components
// which import from "$lib/utils.js"
export { cn } from './utils/cn.js';

import type { Snippet } from 'svelte';

// Type helpers for shadcn-svelte components

/** Component props with an optional element ref. */
export type WithElementRef<
	T,
	El extends HTMLElement = HTMLElement,
> = T & {
	ref?: El | null;
};

/** Omit the `children` snippet prop (for components using named snippets only). */
export type WithoutChild<T> = Omit<T, 'child'>;

/** Omit the `children` snippet prop (plural). */
export type WithoutChildren<T> = Omit<T, 'children'>;

/** Omit both `children` and `child` snippet props. */
export type WithoutChildrenOrChild<T> = Omit<T, 'children' | 'child'>;
