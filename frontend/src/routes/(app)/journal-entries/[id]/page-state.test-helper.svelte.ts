/**
 * Revue P1 de la Story 29-1b (B-1, E-3) — un `page` de `$app/state` **réactif**,
 * pour les seuls tests de la fiche d'écriture.
 *
 * SvelteKit réutilise le composant quand on navigue d'une écriture à l'autre :
 * seul `page.params.id` change, et le `$effect` de la page relit la fiche. Un
 * objet simple dans `vi.mock` ne déclenche rien ; cet état-ci, si — c'est ce
 * qui permet de tester une navigation (confirmation ouverte, réponse tardive de
 * la liste des exercices).
 *
 * ⚠️ Module de test : il n'est importé que par `journal-entry-page.test.ts`.
 */
export const pageState = $state({ params: { id: '7' } as Record<string, string> });
