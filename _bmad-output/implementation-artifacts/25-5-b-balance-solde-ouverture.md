# Story 25.5-b : La balance des comptes porte un solde d'ouverture — et concorde enfin avec le bilan

Status: ready-for-dev

**Issue : [#385]**, que cette story **ferme** : la PR porte `closes #385` dans le **titre ET le corps**. Seconde moitié de
la 25-5, découpée le 2026-09-23 (la 25-5-a a fermé #386). Branche `story/25-5-b-balance-solde-ouverture`, partie de
`main` (`27c61def`) : la story ne dépend pas de la chaîne 25-4, qui ne touche pas `trial_balance.rs`.

## Story

En tant que **personne qui tient ses livres dans Kesh, ou qui les révise**,
je veux que **la balance des comptes montre pour chaque compte son solde d'ouverture, ses mouvements et son solde de
clôture**,
afin que **le solde de clôture d'un compte de bilan soit le même nombre que celui du bilan** — un compte, un nombre.

## Les faits, vérifiés dans le code

- **La balance actuelle est une balance des mouvements.** `crates/kesh-report/src/trial_balance.rs:60-104` filtre les
  lignes sur `je.fiscal_year_id = ?` **et** `je.entry_date BETWEEN start AND end`. `balance` est le **net des mouvements**
  de la période, signé par type (`Asset`/`Expense` : débit − crédit ; sinon crédit − débit).
- **Le bilan est cumulatif depuis l'origine** (`balance_sheet.rs:1-30`) : comptes de bilan = Σ de toutes les écritures
  `entry_date ≤ fin`, tous exercices confondus ; plus deux **lignes calculées** : `retained_earnings` (P&L
  `entry_date < fy_start`) et `equity_result` (P&L `[fy_start, fin]`).
- ⚠️ **L'écran avoue l'écart** : `TrialBalanceView.svelte:26-32` affiche `reports-trial-balance-period-note` — « Le total
  par compte n'est pas comparable au solde cumulé du même compte au bilan ». Cette note devient **fausse** avec la story.
- **Le moteur existe — dans le grand livre** (Story 24-1). `general_ledger.rs:11-35` pose **la règle d'ouverture** :
  `Asset`/`Liability` = cumul `entry_date < from`, tous exercices ; `Revenue`/`Expense` = cumul depuis le **début de
  l'exercice contenant `from`** (0 si `from` est ce premier jour). Implémentée par `opening_balance` (`:220-275`), avec
  `is_debit_natured` / `signed` (`:182-192`) et `fiscal_year_start_containing` (`:194-217`) — **privés au module**, et
  **requête par compte**.
- **Un test de concordance existe déjà** entre grand livre et balance (`crates/kesh-report/tests/general_ledger.rs:150-205`) :
  il ne compare que `total_debit` / `total_credit` — les mouvements, la seule chose que la balance porte aujourd'hui.
- **Règle d'inclusion actuelle** (`trial_balance.rs:3-6`) : comptes actifs toujours ; archivés **seulement s'ils ont une
  écriture dans la période**. Un compte archivé portant un solde d'ouverture mais sans mouvement **disparaîtrait** —
  alors que le bilan le compte.
- **Rendus** : CSV `csv.rs:249-296` (`NumeroCompte;NomCompte;TotalDebit;TotalCredit;Solde`, ligne `Total`) ; PDF
  `pdf.rs:699-850` (5 colonnes, A4 portrait, nom tronqué à 30 caractères, positions `MARGIN_LEFT_MM + 20/90/125/160`).
  Les libellés `ledger_opening`, `ledger_closing` et `retained_result_label` existent déjà dans `SectionLabels`
  (`pdf.rs:74-140`) ; côté écran, `reports-ledger-opening`, `reports-ledger-closing` et
  `reports-retained-earnings-calculated` / `-loss` existent déjà dans les 4 locales.
- **Consommateurs de `TrialBalance` / `TrialBalanceRow`** (construction littérale à compléter) : `csv.rs` (tests
  `:820`, `:1128`), `pdf.rs` (`fixture_tb`, `:1810`), `benches/export.rs:100-125`. Appels de `generate` :
  `kesh-api/src/routes/reports.rs:399`, `:991` ; tests `kesh-db/tests/report_aggregates.rs` (4 sites),
  `kesh-api/tests/reports_e2e.rs`, `reports_export_e2e.rs`, `kesh-report/tests/general_ledger.rs`.
- **Le manuel est faux, et pas seulement sur l'ouverture** : `user-manual.tex:1557-1559` — « Liste tous les comptes avec
  leurs soldes débiteurs et créditeurs. Filtres : période, **niveau de détail (comptes principaux uniquement, ou tous
  sous-comptes)** ». Aucun filtre de niveau n'existe (`grep -n "detail\|niveau" reports/+page.svelte routes/reports.rs` :
  rien). Glossaire `:2043` : « … à une date donnée ».

## Acceptance Criteria

**AC 1 — Le solde d'ouverture, selon la règle du grand livre.** `TrialBalanceRow` gagne `opening_balance` et
`closing_balance` (`Decimal`, même convention de signe que `balance`). Ouverture : comptes de bilan (`Asset`,
`Liability`) = Σ des lignes `entry_date < period.start_date`, **tous exercices confondus** ; comptes de résultat
(`Revenue`, `Expense`) = Σ des lignes `entry_date ∈ [début de l'exercice de la période, period.start_date[` — 0 quand la
période commence le premier jour de l'exercice. Le début de l'exercice est celui de `period.fiscal_year_id` (la
`ReportPeriod` est déjà bornée à cet exercice). **Clôture = ouverture + net des mouvements** (`balance`).

**AC 2 — Une seule règle, pas deux copies.** `is_debit_natured`, `signed` et la règle de borne basse sont **extraits**
de `general_ledger.rs` dans un module partagé de `kesh-report` (ex. `crate::opening`) que les deux rapports appellent ;
le grand livre garde son comportement (ses tests restent verts sans modification de leurs assertions). La balance
calcule ses ouvertures en **une requête agrégée** (pas une requête par compte) : les sommes conditionnelles sur
`entry_date` suffisent.

**AC 3 — Les mouvements ne changent pas.** `total_debit`, `total_credit` (par ligne et au total), `balance` (net des
mouvements) et `balanced` gardent **exactement** leur sens et leurs valeurs actuels. `balance` est documenté au champ
comme « net des mouvements de la période » — l'API publique (clés PAT) ne voit qu'**ajouter** des champs.

**AC 4 — La ligne calculée du résultat reporté.** Sans écriture de clôture, le résultat des exercices antérieurs n'est
porté par **aucun compte** : sans lui, ni la colonne d'ouverture ni celle de clôture ne s'équilibrent. `TrialBalance`
gagne `retained_earnings: Decimal` — **même valeur** que `BalanceSheet::retained_earnings` pour l'exercice de la période
(P&L `entry_date < fy_start`, signe crédit-positif), obtenue en appelant **la même fonction** :
`fetch_retained_earnings` (`balance_sheet.rs:333`, privée) est rendue visible au crate (`pub(crate)`), pas recopiée. Il est rendu comme une **ligne
à part**, après les comptes, libellée « Résultat reporté (calculé) » (« Perte reportée » si négatif, comme au bilan),
ouverture = clôture = `retained_earnings`, mouvements vides. Ce n'est **pas** une `TrialBalanceRow` (pas
d'`account_id`, pas de lien vers le grand livre).

**AC 5 — Un contrôle, pas une tautologie.** `TrialBalance` gagne `opening_balanced` et `closing_balanced` (`bool`) :
Σ des ouvertures (resp. clôtures) **exprimées en sens débit** (positif pour un compte à nature débitrice, négatif sinon)
moins `retained_earnings` vaut zéro. Le résultat reporté est calculé **indépendamment** (comme au bilan), donc
l'égalité est une vérification réelle, pas une identité de construction. Un déséquilibre n'est **pas** une erreur (à la
différence de `TrialBalanceUnbalanced` sur les mouvements) : il est **journalisé** (`tracing::error!`) et affiché (⚠️),
comme `equation_holds` au bilan.

**AC 6 — La règle d'inclusion.** Un compte est rendu s'il est **actif**, **ou** s'il a un mouvement dans la période,
**ou** si son **solde d'ouverture est non nul**. Un compte archivé à solde d'ouverture non nul sans mouvement apparaît
donc (marqueur `active: false`), sinon la clôture ne concorderait pas avec le bilan. Doc du module mise à jour.

**AC 7 — Les rendus.**
- **Écran** (`TrialBalanceView.svelte`) : colonnes N° | Intitulé | **Ouverture** | Débit | Crédit | **Clôture** ; la
  colonne « Solde » (net des mouvements) disparaît de l'écran ; pied : totaux débit/crédit et ✓/⚠️ des mouvements comme
  aujourd'hui, plus un indicateur ✓/⚠️ pour l'ouverture et la clôture (AC 5) ; la ligne calculée (AC 4) avant le pied.
  La note `reports-trial-balance-period-note` est **retirée** (clé supprimée des 4 locales) — elle deviendrait fausse.
- **CSV** : `NumeroCompte;NomCompte;SoldeOuverture;TotalDebit;TotalCredit;SoldeCloture` — la colonne `Solde` est
  **remplacée**, pas renommée en silence (CHANGELOG). Ligne de résultat reporté (libellé fr fixe, comme les autres en-têtes
  CSV), puis ligne `Total` (débit, crédit ; ouverture et clôture vides).
- **PDF** : six colonnes en A4 portrait — nom tronqué plus court (la troncature existante s'adapte), libellés
  `ledger_opening` / `ledger_closing` réutilisés, ligne `retained_result_label`. Garde « vide » inchangée
  (`rows.is_empty()`).
- `isReportEmpty('trial-balance', …)` (`reports.api.ts:169-172`) : inchangé — les comptes actifs sont toujours rendus.

**AC 8 — L'API et les types.** `openingBalance`, `closingBalance` sur chaque ligne ; `retainedEarnings`,
`openingBalanced`, `closingBalanced` au niveau du rapport — dans la réponse de `GET /api/v1/reports/trial-balance` et dans
`reports.types.ts`. Libellés neufs éventuels dans les 4 locales, `sitesTotal` recompté depuis la source.

**AC 9 — La concordance, prouvée en appelant les autres rapports.** Le motif d'échec redouté est **muet** :
`clôture = ouverture + mouvements` resterait vrai, le rapport serait intérieurement cohérent et extérieurement faux.
Les tests appellent donc **réellement** `generate_balance_sheet`, `generate_income_statement` et le grand livre :
- deux exercices, des écritures sur les deux (dont un compte de bilan, un de produits, un de charges) ; balance de
  l'exercice 2 → **clôture de chaque compte de bilan = son solde au bilan** à la fin de la période ; **clôture de chaque
  compte de résultat = son montant au compte de résultat** ; `retained_earnings` = celui du bilan ; ouverture du compte
  de produits **= 0** au premier jour de l'exercice 2 (et non le cumul de l'exercice 1) ;
- une période **en cours d'exercice** (mars–juin de l'exercice 2) : ouverture du compte de produits = cumul janvier–
  février ; clôture = celle du grand livre sur la même période (`section.closing`) ;
- `opening_balanced` et `closing_balanced` vrais dans ces scénarios ;
- un compte **archivé** à solde d'ouverture non nul, sans mouvement → présent, clôture = bilan ;
- non-régression : mouvements, `balance`, `balanced` identiques à avant sur le jeu existant (`report_aggregates.rs`) ;
- le test existant `general_ledger.rs:150-205` étendu à `opening` / `closing`.
- Rendus : CSV (en-tête à six colonnes, ligne de résultat reporté, ligne `Total`) ; PDF (rendu non vide avec la ligne
  calculée) ; **Vitest** de la vue (`TrialBalanceView.test.ts`, à créer : colonnes, ligne calculée, ✓/⚠️, note absente) ;
  test d'API sur la forme de la réponse (`reports_e2e.rs`) ; **E2E** : la balance affiche les colonnes Ouverture /
  Clôture (`reports.spec.ts`, sélecteurs ancrés).

Chacun aurait échoué avant le patch ; chaque test de concordance nomme la mutation qu'il tue (ex. « ouverture des
comptes de résultat cumulée depuis l'origine », « règle d'inclusion sans le solde d'ouverture »).

**AC 10 — Le manuel et le CHANGELOG.** `user-manual.tex` § *Balance des comptes* (`:1557-1559`) réécrit : les quatre
colonnes, la règle d'ouverture (bilan depuis l'origine, résultat depuis le début de l'exercice), la ligne calculée,
la concordance avec le bilan et le grand livre ; la phrase sur le **filtre de niveau de détail**, qui n'existe pas,
**retirée**. Le grand livre (`:1561-1565`, « là où la balance dit la même chose autrement ») relu. Glossaire `:2043`
corrigé (« à une date donnée » → sur une période, ouverture et clôture). PDF régénéré, contrôlé **aplati**
(`pdftotext … | tr '\n' ' ' | tr -s ' '`). CHANGELOG `[Unreleased]` : `Changed` (colonnes de l'écran, du PDF et du CSV —
`Solde` remplacé) et `Fixed` (#385). README et site relus (`grep -rn "balance" README.md website/`).

## Tasks / Subtasks

- [ ] **T1 — le module partagé** (AC 2) : extraire `is_debit_natured`, `signed`, la borne basse de l'ouverture ; le grand
  livre l'appelle ; ses tests verts sans retouche d'assertion.
- [ ] **T2 — la balance** (AC 1, 3, 4, 5, 6) : requête agrégée, champs neufs, ligne calculée, contrôles, inclusion.
- [ ] **T3 — les rendus** CSV et PDF (AC 7), fixtures et bench complétés.
- [ ] **T4 — l'écran, les types, l'i18n** (AC 7, 8) : vue, `reports.types.ts`, clé de note retirée des 4 locales,
  `sitesTotal`.
- [ ] **T5 — tests** (AC 9).
- [ ] **T6 — manuel, CHANGELOG** (AC 10).
- [ ] **T7 — gates** : backend complet, frontend complet, **E2E complet**.

## Dev Notes

### Ce qu'il ne faut pas faire

- **Ne pas** recopier `opening_balance` du grand livre dans `trial_balance.rs` (DRY, AC 2) — ni l'appeler compte par
  compte (une requête par compte sur un plan de 300 comptes).
- **Ne pas** changer le sens de `balance` : l'API est publique (clés PAT, Story 15-x). On **ajoute**.
- **Ne pas** fusionner la ligne calculée avec un compte physique de rôle `RetainedEarnings` (collision D1 de la 14-3c,
  `balance_sheet.rs:40-55`) : le compte physique reste une ligne de compte, la ligne calculée reste à part.
- **Ne pas** borner l'ouverture des comptes de bilan par `fiscal_year_id` : c'est exactement le défaut de #385.
- **Ne pas** introduire de filtre de niveau de détail pour rendre le manuel vrai : c'est le manuel qu'on corrige.

### Limites assumées (à écrire au manuel et en doc de module)

- **Signe lu sur le type courant** du compte : retyper un compte mouvementé re-signe tout son historique — limite
  préexistante du grand livre (#274, #382), héritée telle quelle.
- **Pas d'écriture de clôture** : la remise à zéro des comptes de résultat n'existe que comme borne basse du `SUM` ;
  c'est ce qui rend les tests de concordance (AC 9) indispensables.

### Modules

`kesh-report` (balance, module partagé, rendus), `frontend` (vue, types, test), `kesh-i18n` (clé retirée, libellés
éventuels), + `docs`, `CHANGELOG`. `kesh-api` : aucun changement de code attendu (la route sérialise la structure ;
les libellés PDF existent) — tests seulement. **Trois modules de code**, sous le seuil de la § *Règle de splitting
préventif*.

### References

- Issue [#385] ; audit du 2026-08-26 (comptable § 5.2, DAF § 1.3).
- `crates/kesh-report/src/general_ledger.rs:1-60`, `:182-275` — la règle et son implémentation.
- `crates/kesh-report/src/balance_sheet.rs:1-120`, `:224-228`, `:333` — le calcul de `retained_earnings`.
- Story 24-1 (grand livre) — `_bmad-output/implementation-artifacts/24-1-grand-livre.md`.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- **2026-10-03** — Créée (Guy : « pousse et continue »).

[#385]: https://github.com/guycorbaz/kesh/issues/385
