/**
 * Story 32-8c (refs #723) — la liste des envois du serveur MCP (AC-C2).
 */
import { describe, it, expect, vi, afterEach, beforeEach } from 'vitest';
import { render, fireEvent, cleanup, waitFor } from '@testing-library/svelte';

vi.mock('./mcp-sends.api', () => ({ listMcpSends: vi.fn() }));
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_k: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{ \$(\w+) \}/g, (_m, k) => String(args[k])) : fallback,
	i18nLocale: () => 'fr-CH',
}));

import * as api from './mcp-sends.api';
import McpSendsList from './McpSendsList.svelte';
import type { McpSendSummary } from './mcp-sends.types';

const mockApi = vi.mocked(api);

function ligne(id: number, over: Partial<McpSendSummary> = {}): McpSendSummary {
	return {
		id,
		mcpKeyId: 3,
		mcpKeyName: 'agent',
		mcpKeyRevoked: false,
		toolName: 'kesh_version',
		state: 'pending',
		modified: false,
		expiresAt: '2026-10-27T10:00:00.000',
		decidedAt: null,
		decidedBy: null,
		createdAt: '2026-10-26T10:00:00.000',
		...over,
	};
}

beforeEach(() => vi.clearAllMocks());
afterEach(() => cleanup());

describe('McpSendsList', () => {
	it('état vide : « Aucun envoi »', async () => {
		mockApi.listMcpSends.mockResolvedValueOnce([]);
		const r = render(McpSendsList);
		expect((await r.findByTestId('mcp-sends-empty')).textContent).toContain('Aucun envoi');
	});

	it('un badge par état, dans l’ordre du serveur, la clé révoquée nommée, un lien par envoi', async () => {
		mockApi.listMcpSends.mockResolvedValueOnce([
			ligne(5),
			ligne(4, { state: 'approved' }),
			ligne(3, { state: 'approved', modified: true }),
			ligne(2, { state: 'refused', mcpKeyRevoked: true }),
			ligne(1, { state: 'expired' }),
		]);
		const r = render(McpSendsList);
		await r.findByTestId('mcp-sends-table');
		const etats = [5, 4, 3, 2, 1].map((id) => r.getByTestId(`mcp-sends-state-${id}`).textContent?.trim());
		expect(etats).toEqual(['En attente', 'Validé', 'Validé — modifié', 'Refusé', 'Échu']);
		const lignes = [...r.getByTestId('mcp-sends-table').querySelectorAll('tbody tr')].map((tr) =>
			tr.getAttribute('data-testid'),
		);
		expect(lignes).toEqual([5, 4, 3, 2, 1].map((id) => `mcp-sends-row-${id}`));
		expect(r.getByTestId('mcp-sends-row-2').textContent).toContain('révoquée');
		expect(r.getByTestId('mcp-sends-row-5').textContent).not.toContain('révoquée');
		expect(r.getByTestId('mcp-sends-open-5').getAttribute('href')).toBe('/admin/mcp/sends/5');
	});

	it('Actualiser relit la liste', async () => {
		mockApi.listMcpSends.mockResolvedValueOnce([]);
		const r = render(McpSendsList);
		await r.findByTestId('mcp-sends-empty');
		mockApi.listMcpSends.mockResolvedValueOnce([ligne(9)]);
		await fireEvent.click(r.getByTestId('mcp-sends-refresh'));
		await waitFor(() => expect(mockApi.listMcpSends).toHaveBeenCalledTimes(2));
		expect(await r.findByTestId('mcp-sends-row-9')).toBeTruthy();
	});

	it('refus de l’API (403) : montré', async () => {
		mockApi.listMcpSends.mockRejectedValueOnce({ status: 403, code: 'FORBIDDEN', message: 'Accès refusé.' });
		const r = render(McpSendsList);
		expect((await r.findByTestId('mcp-sends-error')).textContent).toContain('Accès refusé.');
	});
});
