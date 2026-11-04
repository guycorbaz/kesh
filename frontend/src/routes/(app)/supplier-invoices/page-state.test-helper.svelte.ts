/**
 * v0172-correctifs (#870, AC-B2 bis) — un `page` de `$app/state` **réactif**, pour
 * les seuls tests de la liste des factures fournisseurs, sur le modèle de
 * `journal-entries/[id]/page-state.test-helper.svelte.ts`.
 *
 * La liste lit son filtre et sa page dans l'adresse (`page.url.searchParams`) par
 * des `$derived` : un objet simple dans `vi.mock` ne les rejouerait pas, cet
 * état-ci, si. Le simulacre de `goto` **réaffecte** `pageState.url` (un `URL`
 * muté n'est pas suivi par `$state`), et le `beforeEach` du test la remet à
 * `/supplier-invoices` : l'état du module est partagé par les tests du fichier.
 *
 * ⚠️ Module de test : il n'est importé que par `supplier-invoices-list-page.test.ts`.
 */
export const pageState = $state({ url: new URL('http://localhost/supplier-invoices') });
