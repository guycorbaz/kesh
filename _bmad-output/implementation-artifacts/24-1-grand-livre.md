# Story 24.1 : Le grand livre — savoir de quoi un solde est fait

## Status

ready-for-dev

## Story

**As a** personne qui tient des comptes dans Kesh — ou la fiduciaire qui les relit,
**I want** ouvrir un compte et voir, ligne à ligne, ce qui a fait son solde,
**so that** je puisse justifier ce solde, repérer une écriture aberrante, et répondre à la
première demande d'une révision.

Première story de la **Vague 1** du plan d'action de l'audit du 2026-08-26. Couvre les
issues **#373** (le grand livre) et **#374** (le filtre par compte).

⚠️ **Spécification établie par un expert-comptable suisse** consulté le 2026-08-26, sur
arbitrage du Project Lead : spec courte plutôt que boucle adversariale, le rapport étant en
lecture seule, sans écriture ni migration.

## Pourquoi cette story passe en tête de la vague

**Les trois experts la citent ; deux la placent en tête de leurs priorités, indépendamment
l'un de l'autre.** L'argument de l'expert-comptable : *« c'est l'instrument qui rend les
autres défauts visibles »*. Sans grand livre, on ne peut ni détecter une écriture aberrante,
ni mesurer l'écart entre le compte débiteurs et la balance âgée, ni contrôler la TVA, ni
préparer un bouclement — ni, plus tard, **vérifier que l'écriture d'encaissement fonctionne**.

> La balance dit que le compte débiteurs vaut 84 320.15. **Rien dans Kesh ne permet
> aujourd'hui de savoir de quoi c'est fait.**

## Ce qui existe et doit être réutilisé, pas réinventé

| brique | où |
|---|---|
| SQL **cumulatif depuis l'origine** (`je.entry_date <= ?`, sans `fiscal_year_id`) | `kesh-report/src/balance_sheet.rs`, `fetch_cumulative_section` |
| jointure lignes/comptes et rendu | `kesh-report/src/journal_report.rs` |
| conventions CSV (BOM, `;`, CRLF, `format_amount_iso`) | `kesh-report/src/csv.rs` |
| patron PDF | `kesh-report/src/pdf.rs`, `render_journal_report_pdf` |
| pagination (`offset`/`limit`, `MAX_LIMIT`) | `kesh-api/src/routes/journal_entries.rs` |
| composant d'écran | `frontend/src/lib/features/reports/TrialBalanceView.svelte` |

## D1 — La règle de frontière, et c'est le cœur de la story

⛔ **Ne PAS réutiliser `ReportPeriod`.** Il porte un `fiscal_year_id` **obligatoire** et refuse
toute borne hors exercice (`period.rs`). Créer `LedgerPeriod { from, to }`, sans exercice.

⚠️ **Le grand livre est le premier rapport de Kesh qui franchit la borne d'exercice, et c'est
délibéré** — sinon il ne concorde pas avec le bilan, qui est cumulatif.

**Le solde d'ouverture se calcule DIFFÉREMMENT selon le type de compte** :

| type | ouverture |
|---|---|
| **Asset, Liability** | Σ sur `entry_date < from`, **tous exercices confondus**, aucune borne basse — patron exact de `fetch_cumulative_section` |
| **Revenue, Expense** | Σ sur `fy_start(from) ≤ entry_date < from`, où `fy_start(from)` est le début de l'exercice contenant `from`. Si `from` est le premier jour d'un exercice → **0**. Si aucun exercice ne contient `from` → `fy_start := from`, donc **0**. |

⚠️ **Pourquoi, et un développeur ne le devinera pas** : un compte de bilan reporte son solde
d'un exercice à l'autre. Un compte de résultat est soldé au bouclement et **repart de zéro** ;
le cumuler depuis l'origine donnerait un nombre qui **ne correspond à rien** — ni au compte de
résultat, ni à la balance, ni au bilan. ⛔ **Kesh ne passant aucune écriture de clôture, cette
remise à zéro n'existe QUE comme borne basse du `SUM` : elle est entièrement à la charge de ce
rapport.**

**Période traversant deux exercices** — comptes de résultat : insérer une **ligne de rupture**
à chaque `fy_start` traversé (« Clôture de l'exercice N — solde viré au résultat ») et
**remettre le solde progressif à zéro** juste après. Sans elle, le solde progressif d'un compte
de charges affiché sur deux ans est un nombre qui ne veut rien dire. Payload :
`fiscalYearBreaks: [{ date, closingFiscalYearId, closingBalance }]`.

## D2 — Le tri, et le piège du numéro de pièce

```sql
ORDER BY je.entry_date ASC, je.fiscal_year_id ASC, je.entry_number ASC,
         jel.line_order ASC, jel.id ASC
```

⚠️ **`fiscal_year_id` AVANT `entry_number`** : le numéro de pièce est
`UNIQUE (company_id, fiscal_year_id, entry_number)` — il **repart à 1 à chaque exercice**
(vérifié). Sans ce tri, deux écritures de même date sur deux exercices s'entrelacent.
`jel.id` en dernier ressort pour un ordre **totalement déterministe** — c'est testable.

## D3 — Le solde progressif reprend la convention existante

⛔ **Aucune nouvelle convention.** Celle de la balance et du bilan :

| type | solde progressif |
|---|---|
| Asset, Expense | `précédent + débit − crédit` |
| Liability, Revenue | `précédent + crédit − débit` |

Exposer `balanceSide: "debit" | "credit"` pour l'affichage, et **`unnaturalBalance: true`**
quand le solde de clôture est négatif : *un compte de produits à solde débiteur est exactement
l'anomalie que ce rapport doit rendre visible.*

## Critères d'acceptation

**AC1** (porte **D1**) — L'ouverture d'un compte **de bilan** cumule depuis l'origine, tous
exercices confondus. Celle d'un compte **de résultat** part du début de son exercice.

**AC2** (porte **D1**) — Sur une période traversant deux exercices, un compte de résultat
porte une **ligne de rupture** et son solde progressif **repart de zéro**.

**AC3** (porte **D2**) — L'ordre des lignes est **stable entre deux appels**, et deux écritures
de même date sur deux exercices différents ne s'entrelacent pas.

**AC4** — Chaque section de compte porte **trois lignes d'encadrement** : « Solde au *from* »,
« Total des mouvements » (Σdébit, Σcrédit), « Solde au *to* ». ⚠️ **Sans ces trois lignes, ce
n'est pas un extrait de compte.**

**AC5** — Colonnes : date, **pièce** (avec l'exercice si la période en traverse plusieurs),
journal, **libellé de l'écriture** — ⚠️ `journal_entry_lines` **n'a aucun libellé par ligne**,
vérifié —, **contrepartie**, débit, crédit, solde progressif.

**AC6** — ⛔ **Un compte ARCHIVÉ à solde résiduel est INCLUS**, avec `active: false` visible.
*C'est précisément l'anomalie recherchée.* ⚠️ **Ne pas reproduire l'exclusion de
`trial_balance.rs`**, qui filtre sur `a.active = TRUE`.

**AC7** — Un compte **sans mouvement mais à ouverture ≠ 0** rend une section vide avec
`opening == closing`. **Non négociable** : un compte qui porte encore un solde doit se voir.

**AC8** — ⛔ **`opening`, `closing` et les totaux se calculent sur la PÉRIODE, jamais sur la
page.** Trois requêtes distinctes : ouverture, page de mouvements, totaux.

**AC9** — La route est scopée sur `company_id`, **sur `a.company_id` ET `je.company_id`**, et
montée **à l'intérieur** du routeur authentifié. ⚠️ Une route orpheline est un bypass d'auth —
le dépôt porte déjà un commentaire d'alerte à ce sujet dans `kesh-api/src/lib.rs`.

**AC10** — Sorties : écran, **CSV** (ouverture et clôture en **lignes libellées**, pas en
métadonnées hors tableau — un CSV doit rester réconciliable dans un tableur) et **PDF** (en-tête
de compte **répété à chaque page**, « À reporter » en bas, « Report » en haut de la suivante —
c'est l'usage suisse, et sans lui un extrait multi-pages est illisible).

**AC11** — ⛔ **Un lien depuis chaque ligne du bilan et de la balance vers le grand livre du
compte.** C'est ce lien qui répond à *« la balance dit 84 320.15, de quoi c'est fait »* — sans
lui, le rapport existe mais personne ne le trouve.

**AC12** — i18n : préfixe `reports-ledger-*`, **quatre locales** dès l'écriture. Réutiliser les
clés existantes `reports-column-*`.

## Tasks

- [x] **T1** — `LedgerPeriod` + module `kesh-report/src/general_ledger.rs`. ⛔ L'ouverture passe
      par **le même point** que `fetch_cumulative_section`, pas par une copie du SQL — c'est
      la couture par laquelle le snapshot de clôture (#270) se branchera plus tard.
- [x] **T2** — Contrepartie : **seconde requête** `WHERE jel.entry_id IN (…)` sur les entrées
      retenues, agrégation en Rust. ⛔ Pas de sous-requête corrélée par ligne.
- [x] **T3** — Route `GET /api/v1/reports/general-ledger` + `/export`, paginée.
- [x] **T4** — Écran `GeneralLedgerView.svelte`, et **les liens depuis le bilan et la balance**
      (AC11).
- [x] **T5** — CSV et PDF (AC10).
- [x] **T6** — i18n, quatre locales.
- [x] **T7** — Tests : les invariants ci-dessous, **écrits en même temps que le générateur**.

## Invariants testables — ce qui protège le rapport dans le temps

**Par section** : `closing == opening ± (Σdébit − Σcrédit)` selon le type ; le solde progressif
de la dernière ligne `== closing` ; section sans mouvement ⇒ `opening == closing`.

**Concordance avec l'existant** — ⛔ **ces trois-là appellent RÉELLEMENT les générateurs
existants sur le même jeu de données** :

1. compte de bilan : `closing` au `to` **==** la `balance` du même compte dans
   `generate_balance_sheet(as_of = to)` ;
2. compte de résultat, période = exercice entier : Σdébit/Σcrédit **==** ceux de
   `generate_trial_balance`, et `closing` **==** le montant dans `generate_income_statement` ;
3. compte de bilan, période = exercice entier : Σdébit/Σcrédit **==** ceux de `trial_balance`
   — ⚠️ **sur les MOUVEMENTS seulement, pas sur le solde** : la balance est une balance de
   mouvements. Ce test **documente** l'écart au lieu de le masquer (cf. #385).

**Sur le livre complet** : Σ tous débits `==` Σ tous crédits — c'est la partie double, et le
test le moins cher du lot. `from` = origine ⇒ toutes les ouvertures valent 0. Ordre stable
entre deux appels. ⛔ **`limit=5` et `limit=1000` rendent le MÊME `opening`, le MÊME `closing`
et les MÊMES totaux** — c'est le test qui attrape le piège d'AC8.

⚠️ **Un test à NE PAS écrire** : les ouvertures **signées par type** ne somment pas à zéro.
C'est en **brut** (`Σdébit − Σcrédit`, sans le signe) que la somme sur tous les comptes vaut 0
à n'importe quelle date.

## Hors périmètre, délibérément

Écritures d'à-nouveau réelles et bouclement (#232) — le report reste virtuel ; snapshot des
soldes de clôture (#270) ; comparaison N-1 et vue pluriannuelle (#404) ; ventilation par projet
— déjà couverte par les deux rapports projet ; **lettrage** — Epic 15 gelé, le grand livre
l'affichera plus tard sans le définir ; multidevise ; recherche plein texte.

⚠️ **Le retypage d'un compte** (#274, #382) : le signe est lu sur le `account_type` **courant**,
donc tout l'historique est re-signé silencieusement. **Hors périmètre, mais À DOCUMENTER dans
le doc-comment du module** — sinon quelqu'un inventera un correctif local.

## Dev Notes

⛔ **Ce qui coûterait le plus cher, pris de travers : la règle du solde d'ouverture (D1) — et
sa forme dangereuse est le SILENCE.**

Si l'ouverture est bornée par `fiscal_year_id`, ou si un compte de résultat est cumulé depuis
l'origine, **rien ne rougit** : `closing = opening + mouvements` reste vrai, les totaux
s'additionnent, la page s'affiche, le PDF sort. **Le rapport est intérieurement cohérent et
extérieurement faux.** Le seul moyen de s'en apercevoir serait de rapprocher à la main le grand
livre du bilan — c'est-à-dire exactement le rapprochement que ce rapport existe pour rendre
inutile.

⚠️ **C'est le mode d'échec du test muet, transposé au métier.** Et la correction serait chère,
car la règle ne vit pas à un seul endroit : SQL, payload JSON, contrat TypeScript, colonnes
CSV, mise en page PDF, jeux de test — et, sitôt livré, captures d'écran et manuel.

**D'où l'exigence de T7** : les trois invariants de concordance s'écrivent **en même temps que
le générateur**, pas après.

⚠️ **La base de gate se remet à zéro AVANT le gate**, inconditionnellement (KF-039, #310).

## Change Log

### Spécification — 2026-08-26 (expert-comptable suisse)

Établie sur consultation d'un expert-comptable, à la demande du Project Lead. **Trois
affirmations vérifiées au sol par l'orchestrateur avant rédaction** : le numéro de pièce repart
bien à 1 à chaque exercice (`uq_journal_entries_number`), `journal_entry_lines` n'a
effectivement **aucun libellé**, et `trial_balance` exclut bien les comptes archivés — exclusion
que cette story demande de **ne pas** reproduire.

### Implémentation — 2026-08-26

**Livré** : module `crates/kesh-report/src/general_ledger.rs`, routes
`GET /api/v1/reports/general-ledger` et `…/export?format=pdf|csv`, écran
`GeneralLedgerView.svelte` avec son onglet, liens depuis le bilan et la balance, rendus CSV et
PDF, 24 clés `reports-ledger-*` dans les quatre locales.

**Décomptes, recomptés depuis la source et non repris d'une passe antérieure** — périmètre
`main…HEAD` de la branche `story/vague1-grand-livre` :

| Mesure | Valeur | Commande |
|---|---|---|
| tests d'intégration du grand livre | 10 | `grep -c '^#\[sqlx::test' crates/kesh-report/tests/general_ledger.rs` — ⚠️ **pas** `grep -c '^async fn'`, qui rend 11 en comptant le helper `post` |
| tests unitaires PDF du grand livre | 3 | `grep -c 'fn general_ledger_pdf' crates/kesh-report/src/pdf.rs` |
| tests unitaires CSV du grand livre | 2 | `grep -c 'fn general_ledger_csv' crates/kesh-report/src/csv.rs` |
| tests Vitest de l'écran | 8 | `npx vitest run …/GeneralLedgerView.test.ts` |
| clés i18n `reports-ledger-*` par locale | 24 | `grep -c '^reports-ledger-' …/messages.ftl` |
| sites `i18nMsg` ajoutés | 33 | ventilés dans `i18n-keys.test.ts` |

**Trois écarts à la spec, corrigés en cours de route et signalés ici parce qu'aucun test ne les
aurait attrapés seuls** :

1. **AC5 — la pièce sans son exercice.** Le premier écran affichait `entryNumber` nu. Comme ce
   numéro **repart à 1 à chaque exercice**, un extrait couvrant deux exercices contenait deux
   « pièce n° 12 » indiscernables. Corrigé par un `fiscal_year_name` joint depuis
   `fiscal_years`, rendu en préfixe **seulement** quand la période traverse une rupture — le
   préfixe systématique serait du bruit. La colonne « Exercice » du CSV portait, elle,
   l'**identifiant** de l'exercice : un id de base de données ne dit rien à qui lit le fichier.

2. **AC10 — un export tronqué.** `LedgerOptions::limit = None` valait « le défaut, soit 500 »,
   si bien que l'export était plafonné comme l'écran — alors que le bandeau de l'écran promet
   « l'export les contient toutes ». **C'est la documentation qui promettait ce que le code ne
   faisait pas**, exactement le défaut relevé en vague 0. `None` veut désormais dire « aucune
   borne », et deux tests le tiennent : l'export rend `MAX + 3` lignes là où l'écran s'arrête à
   `MAX`, et une limite explicite trop grande reste écrêtée — `None` est le seul moyen de lever
   la borne.

3. **Trois erreurs `clippy -D warnings`** dont une `items_after_test_module` : le rendu CSV
   avait été écrit **après** `mod tests`. Déplacé avant.

**Gates** : `cargo fmt --check` vert, `cargo clippy --workspace --all-targets -D warnings` vert,
`scripts/test-fast.sh` complet vert, base de gate remise à zéro avant le run (KF-039).
`npm run check` 0 erreur, `lint-i18n-ownership` vert, `npm run test:unit` vert, `npm run build`
vert.
