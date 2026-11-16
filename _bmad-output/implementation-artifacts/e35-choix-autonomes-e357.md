# Epic 35 — registre des choix autonomes de la story 35-7 (clé `e357`)

Registre propre à la story 35-7 (#1000), **versé par union** par l'orchestrateur. Chaque choix : contexte, option
retenue, options écartées, réversibilité. Lignes citées sur la branche `story/35-7-autocomplete-compte-hors-liste`.

---

### C-e357-1 — Les tests d'AC3 montent les dialogues par un hôte, pas par `rerender`

- **Contexte** : AC3 demande de monter le dialogue avec `bankLedgerAccountId: null`, de choisir le compte de la banque,
  puis de passer le prop à son identifiant « (`rerender`) ». Joué tel quel, `rerender` de `@testing-library/svelte`
  5.3.1 remplace l'objet de props entier (`svelte-core/src/props.svelte.js`, `$state.raw`) : chaque prop paraît
  changée, l'effet de remise à zéro du dialogue (`open && proposal`) se rejoue et vide la sélection **quel que soit le
  code**. Constaté au premier rouge : le test « banque → champ vide » d'*Écriture manuelle* passait à vide, et le test
  « compte resté proposé » rougissait sans raison. Même piège déjà écrit par la 15-6b
  (`SettleInvoiceDialogHost.test.svelte`).
- **Retenu** : un hôte de test, `CounterpartyModalHost.test.svelte`, qui passe les props une à une comme
  `ReconciliationProposals` et ne change que `bankLedgerAccountId` (bouton `host-resolve-bank`).
- **Écartées** : (a) `rerender` avec le même objet `proposal` : la remise à zéro dépend du signal de l'objet de props,
  pas de l'égalité des valeurs ; (b) déplacer la remise à zéro du dialogue : hors périmètre, touche le comportement.
- **Réversible** : oui (fichier de test seul).

### C-e357-2 — Le mock d'`i18nMsg` de `TransactionSplitModal.test.ts` interpole ses arguments

- **Contexte** : AC3 attend « Ligne 1 : compte requis » et « Ligne 2 : compte requis ». Le mock du fichier rendait le
  repli brut, « Ligne { $line } : compte requis », qui ne distingue pas les deux lignes.
- **Retenu** : le mock interpolant de `ContactPersonsManager.test.ts`, pour tout le fichier (aucune assertion existante
  ne dépend d'un `{ $…}` brut, `grep '{ \$'` vide sur le fichier).
- **Écartées** : asserter le repli brut (ne prouve pas la ligne nommée) ; un second fichier de test (duplication des
  fixtures).
- **Réversible** : oui.

### C-e357-3 — AC4 : un test ajouté dans `JournalEntryForm.edit.test.ts`

- **Contexte** : AC4 laisse le choix entre un test de la ligne d'écriture et le constat que le composant seul suffit.
- **Retenu** : le test (vider le compte de la ligne 1, quitter, le libellé revient ; le PUT porte l'identifiant). Il
  tue M1 et M2 au niveau du formulaire. `VatPurchaseAssistant` n'a pas de suite de composant : ses deux champs stricts
  sont couverts par le composant seul (AC1).
- **Réversible** : oui.

### C-e357-4 — La capture d'*Éclater* (AC6) par une spec jetable hors dépôt

- **Contexte** : aucune spec E2E ne parcourt *Éclater* (`reconciliation-split.spec.ts` ne fait qu'un smoke et un scan
  axe) ; AC7 n'en demande pas. La capture d'*Écriture manuelle* vient de l'E2E d'AC7.
- **Retenu** : une spec et une config jetables dans `frontend/.output/e357/` (ignoré par git), jouées une fois contre
  la base jetable `kesh_e2e_e357`.
- **Écartées** : ajouter un parcours *Éclater* aux specs versionnées (hors AC, et une spec de plus au gate de chacun) ;
  une issue est proposée au rapport pour une couverture E2E d'*Éclater*.
- **Réversible** : oui.
