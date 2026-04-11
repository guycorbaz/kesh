import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { debounce } from './debounce';

describe('debounce', () => {
	beforeEach(() => {
		vi.useFakeTimers();
	});

	afterEach(() => {
		vi.useRealTimers();
	});

	it("n'appelle la fonction qu'une fois après un burst", () => {
		const fn = vi.fn();
		const debounced = debounce(fn, 300);

		debounced();
		debounced();
		debounced();

		expect(fn).not.toHaveBeenCalled();

		vi.advanceTimersByTime(299);
		expect(fn).not.toHaveBeenCalled();

		vi.advanceTimersByTime(1);
		expect(fn).toHaveBeenCalledTimes(1);
	});

	it('réinitialise le timer à chaque appel', () => {
		const fn = vi.fn();
		const debounced = debounce(fn, 300);

		debounced();
		vi.advanceTimersByTime(200);
		debounced();
		vi.advanceTimersByTime(200);
		debounced();
		vi.advanceTimersByTime(200);
		expect(fn).not.toHaveBeenCalled();

		vi.advanceTimersByTime(100);
		expect(fn).toHaveBeenCalledTimes(1);
	});

	it('passe les arguments au dernier appel', () => {
		const fn = vi.fn();
		const debounced = debounce(fn, 100);

		debounced(1, 'a');
		debounced(2, 'b');
		debounced(3, 'c');

		vi.advanceTimersByTime(100);
		expect(fn).toHaveBeenCalledWith(3, 'c');
		expect(fn).toHaveBeenCalledTimes(1);
	});

	it('cancel() empêche l\'appel', () => {
		const fn = vi.fn();
		const debounced = debounce(fn, 300);

		debounced();
		debounced.cancel();
		vi.advanceTimersByTime(500);

		expect(fn).not.toHaveBeenCalled();
	});

	it('cancel() après l\'appel est un no-op', () => {
		const fn = vi.fn();
		const debounced = debounce(fn, 100);

		debounced();
		vi.advanceTimersByTime(200);
		expect(fn).toHaveBeenCalledTimes(1);

		expect(() => debounced.cancel()).not.toThrow();
	});

	it('peut être rappelé après cancel()', () => {
		const fn = vi.fn();
		const debounced = debounce(fn, 100);

		debounced();
		debounced.cancel();
		debounced();
		vi.advanceTimersByTime(100);

		expect(fn).toHaveBeenCalledTimes(1);
	});
});
