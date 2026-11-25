# Story 55-2 : le lien de compte partout — Journaux, Grand livre, TVA, Compte de résultat

Status: drafted

Epic : `_bmad-output/planning-artifacts/epic-55-navigation-interface.md` — D6, D7 (Q3), D8. Issue : #À-VENIR (`enhancement`,
P4, point a). Release : la prochaine planifiée après la 0.20.0. **Frontend seulement, aucune migration. Suit la 55-1**
(l'URL du grand livre et le Retour).

## Story

En tant que comptable,
je veux que chaque numéro de compte d'un rapport ouvre le grand livre de ce compte sur la période du rapport, de la même
façon partout,
afin de passer d'une écriture du journal, d'une contrepartie ou d'une ligne de TVA à l'extrait du compte, et d'en revenir
par *Retour*.

## Modules touchés (règle de splitting préventif)

Quatre : `frontend/src/lib/features/reports` (composant neuf, sept vues), `frontend/src/routes/(app)/reports` (données
passées aux vues), `kesh-i18n` (quatre locales), `docs/manual/fr/user-manual.tex` ; plus `frontend/tests/e2e`. Aucun Rust.

## Ce qui se lit au code

Relevé à `origin/main` `1e23edb55` et `pr/46-rapports-groupes` `511e61d54` ; lignes de la PR 46 relevées au T0.

| Vue | Numéro de compte affiché | Lien aujourd'hui | Période du lien |
|---|---|---|---|
| `BalanceSheetView.svelte` (actifs `:140`, passifs `:179`, fonds propres `:234`) | oui | `<a href={ledgerHref(…)}>` recopié **trois fois** (`:143-149`, `:182-188`, `:237-243`) | `dto.period.startDate`/`endDate` |
| `TrialBalanceView.svelte:67-72` | oui | même `<a>` recopié | idem |
| `GroupedReportRows.svelte` (PR 46, `:150-154`) | oui | même `<a>`, lignes `account` et `own` ; ligne `group` : bouton d'accordéon, pas de lien (à garder) | `period` (prop) |
| `IncomeStatementView.svelte:60`, `:90` (à plat) | oui | **aucun** (texte) — ni à `main` ni sur la PR 46 | — |
| `JournalReportView.svelte:68` | oui, `l.accountNumber`, avec `l.accountId` | **aucun** | `dto.period.startDate`/`endDate` |
| `GeneralLedgerView.svelte:220` (titre de section) | oui, `s.accountNumber`, `s.accountId` | **aucun** | `dto.period.from`/`to` (`LedgerPeriod`) |
| `GeneralLedgerView.svelte:289` (contrepartie) | oui, **numéros seuls** (`counterpart: string[]`, `reports.types.ts:342`) | **aucun** | idem |
| `VatReportView.svelte` | **aucun** (lignes par taux, totaux) | — | `dto.period` |

- Le titre `i18nMsg('reports-ledger-open-from-balance', 'Voir le détail dans le grand livre')` est repris tel quel (une
  clé, quatre locales déjà traduites).
- La page charge déjà la liste des comptes, archivés compris (`fetchAccounts(true)`, `+page.svelte:129-133`), pour le grand
  livre et le lettrage ; le numéro est unique par société (`uq_accounts_company_number`,
  `migrations/20260411000001_accounts.sql:19`) : il désigne le compte sans ambiguïté.
- Les comptes de TVA du décompte : `vat_report.rs:90` lit `default_vat_recoverable_account_id` ; la TVA due se rapproche
  du compte TVA due (`:98`). Côté écran, `getInvoiceSettings()` (`invoices.api.ts:99`, `GET
  /api/v1/company/invoice-settings`, **tout rôle authentifié**, `kesh-api/src/lib.rs:1309`) rend
  `defaultVatPayableAccountId` et `defaultVatRecoverableAccountId` (`invoices.types.ts:144-145`).
- `ledgerHref` (55-1) : sortie inchangée, écrite sur la fonction d'URL de la 55-1.

## Critères d'acceptation

1. **AC-1 — Un composant.** `frontend/src/lib/features/reports/AccountLink.svelte` (neuf) rend le numéro en lien
   `ledgerHref(accountId, from, to)`, avec le titre d'aujourd'hui, la classe d'aujourd'hui (`text-indigo-700
   hover:underline`), un `data-testid` passé par la vue ; sans `accountId` (compte inconnu), il rend le numéro en texte. Les
   **cinq** `<a href={ledgerHref…}>` d'aujourd'hui (Bilan ×3, Balance, `GroupedReportRows`) sont remplacés par lui : après
   la story, `grep -rn "ledgerHref(" frontend/src/lib/features/reports/*.svelte` ne rend **que** `AccountLink.svelte`.
   L'accueil et la réévaluation gardent leurs liens (hors Rapports, hors périmètre).
2. **AC-2 — Compte de résultat à plat.** Produits et charges : le numéro devient `AccountLink`, période du rapport.
3. **AC-3 — Journaux.** Chaque ligne d'écriture : `AccountLink` sur `l.accountNumber` / `l.accountId`, période du journal.
4. **AC-4 — Grand livre, contreparties (D8).** Chaque numéro de `counterpart` devient un lien vers le grand livre de ce
   compte, **même période** (`dto.period.from`/`to`) ; le numéro se résout en compte par la liste des comptes de la page
   (`Map` numéro → identifiant, construite une fois) ; numéro absent de la liste (liste en échec) : texte, sans erreur.
   Les numéros restent séparés par « , » comme aujourd'hui.
5. **AC-5 — Grand livre, titre (D8).** Le numéro du titre d'une section est un lien **seulement** quand le grand livre
   montre plusieurs sections (« Tous les comptes mouvementés ») ; filtré sur un compte, il reste du texte.
6. **AC-6 — TVA (D7, Q3).** Sous le tableau du décompte, une ligne `data-testid="vat-accounts"` : « Comptes au grand
   livre : TVA due `NNNN` · TVA récupérable `MMMM` », chaque numéro en `AccountLink` sur la période du décompte. Les
   comptes viennent de `getInvoiceSettings()` (une requête au chargement de la page) et de la liste des comptes ; un compte
   non désigné, ou des réglages en échec : sa mention est omise ; les deux : la ligne est omise. Aucun changement du DTO de
   la TVA, du PDF ni du CSV.
7. **AC-7 — Retour.** Depuis chacun des liens ci-dessus, *Retour* ramène au rapport d'origine (55-1) : un E2E le constate
   pour le Journal, une contrepartie du Grand livre et la TVA.
8. **AC-8 — Rien d'autre ne bouge.** Exports PDF et CSV, ordre et texte des lignes, montants, accessibilité (axe à zéro
   violation sur `reports.spec.ts`), libellés d'aujourd'hui : inchangés ; les Vitest existants des vues passent sans perte
   d'assertion (ceux qui lisaient un `<td>` de texte sont adaptés au lien, adaptation nommée au Change Log).
9. **AC-9 — Textes.** Clés neuves dans un bloc `reports-nav-*` des quatre locales (`crates/kesh-i18n/locales/{fr,de,it,en}-CH/
   messages.ftl`) : `reports-nav-vat-accounts` (« Comptes au grand livre : »), `reports-nav-vat-payable` (« TVA due »),
   `reports-nav-vat-recoverable` (« TVA récupérable ») — ou réutilisation de `reports-vat-recoverable` si le texte est le
   même, décidé au T0 et écrit au Change Log ; `i18n-keys` (`sitesTotal` recompté par la garde) et `i18n-libelle-en-dur`
   verts.
10. **AC-10 — Manuel.** `docs/manual/fr/user-manual.tex` : § *Grand livre* (`:4184`) « tout numéro de compte d'un rapport
    ouvre ce grand livre », contreparties comprises ; § *Journal des écritures* (`:4316`) et § *Décompte TVA* (`:4320`) :
    une phrase chacun ; la phrase du Bilan ou de la Balance qui ne citait que ces rapports est élargie. `.tex` seul.

## Tests à écrire, et la preuve du rouge

- **Vitest** `AccountLink.test.ts` (neuf) : lien et `href` exacts, titre, texte sans `accountId`.
- **Vitest des vues** : `IncomeStatementView.test.ts`, `JournalReportView.test.ts`, `GeneralLedgerView.test.ts` (contrepartie
  résolue, non résolue, titre lié à plusieurs sections et non lié à une), `VatReportView.test.ts` (deux comptes, un seul,
  aucun), `BalanceSheetView.test.ts` et `TrialBalanceView.test.ts` (lien inchangé). Chacun rougit sans le correctif
  (journal `kesh-gate-logs/s55-2/vitest-rouge-avant.log`), sauf les deux derniers, filets de non-régression déclarés tels.
- **E2E** `frontend/tests/e2e/reports-navigation.spec.ts` (fichier de la 55-1, bloc `55-2`) : Journal → lien d'une ligne →
  grand livre du compte → *Retour* → Journal ; Grand livre « tous les comptes » → contrepartie → grand livre de la
  contrepartie → *Retour* ; TVA → compte TVA due → *Retour*.
- La garde DRY de l'AC-1 se vérifie par la commande `grep` de l'AC, journal au Change Log.

## Recette visuelle

Captures (thème clair, `kesh-gate-logs/s55-2/`) : Journal avec les liens ; Grand livre, contreparties en liens ; titre de
section lié et non lié ; TVA avec sa ligne de comptes ; Compte de résultat à plat.

## Tâches

- [ ] **T0** — Base : `epic/55-navigation` avec la 55-1 ; relever les lignes ; choisir les clés i18n (AC-9).
- [ ] **T1** — Vitest rouges des vues neuves.
- [ ] **T2** — `AccountLink.svelte` ; remplacement des cinq sites.
- [ ] **T3** — Compte de résultat, Journaux, Grand livre (carte numéro → compte passée par la page), TVA (réglages lus par
      la page, une requête, échec silencieux).
- [ ] **T4** — i18n, manuel, E2E, captures.
- [ ] **T5** — Gates frontend ; E2E de la story et des Rapports.

## Gates

`cd frontend && npm run check && npm run lint-i18n-ownership && npm run test:unit && npm run build` ; E2E des Rapports ;
suite complète en fin d'epic.

## Note de version

*Rapports* : le numéro de compte ouvre le grand livre de ce compte partout — Compte de résultat, Journaux, contreparties du
Grand livre et, pour la TVA, les comptes de TVA due et d'impôt préalable. (refs #À-VENIR)

## Dev Notes

### Choix et écarts

- **C-55-2-1** — Un composant plutôt qu'une fonction : le titre, la classe et le repli en texte font partie de ce qui doit
  être identique partout (DRY) ; `ledgerHref` reste la fonction sous-jacente.
- **C-55-2-2** — TVA par les réglages (D7). *Écarté* : un champ de comptes dans le DTO de la TVA (backend, hors de cet
  epic frontend). Le compte de la réconciliation est celui des réglages **du moment**, comme le calcul du décompte
  (`vat_report.rs:90`) : le lien montre ce que le décompte lit. La phrase du manuel le dit.
- **C-55-2-3** — Les numéros de contrepartie se résolvent côté écran (liste déjà chargée) plutôt que d'ajouter les
  identifiants au DTO du grand livre : aucune requête de plus.

### References

- Fiche 55-1 ; fiche 46-2 AC-4 (`ledgerHref` sur `account` et `own` seulement).

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

- 2026-11-25 — Fiche écrite au kickoff de l'Epic 55 (`drafted`).
