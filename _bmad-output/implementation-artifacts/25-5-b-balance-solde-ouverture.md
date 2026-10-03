# Story 25.5-b : La balance des comptes porte un solde d'ouverture — et concorde enfin avec le bilan

Status: done

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
  `is_debit_natured` / `signed` (`:182-193`) et `fiscal_year_start_containing` (`:195-218`) — **privés au module**, et
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
  `reports-retained-earnings-calculated` / `-loss` existent déjà dans les 4 locales. ⚠️ Les libellés « Solde
  d'ouverture » / « Solde de clôture » sont **trop longs** pour les colonnes de la balance (AC 7) : seuls
  `retained_result_label` et `reports-retained-earnings-*` sont réutilisés.
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

**AC 2 — Une seule règle, pas deux copies — et la forme qui le permet.** La règle type → borne vit **en Rust, une
fois** : un module partagé de `kesh-report` (ex. `crate::opening`) porte `is_debit_natured`, `signed` et
`opening_from(t, before_start: (debit, credit), since_fy_start: (debit, credit)) -> Decimal` (bilan : la première
paire ; résultat : la seconde ; signé par type). La balance fait **une requête agrégée** (pas une requête par compte)
qui rend, par compte, **trois paires de sommes brutes** — `entry_date < start`, `entry_date ∈ [fy_start, start[`, et les
mouvements de la période — **sans** `CASE` sur le type ni sur le signe en SQL (le `CASE` de signe actuel,
`trial_balance.rs:71-75`, disparaît au profit de `signed`). Le grand livre appelle `opening_from` lui aussi (sa
requête d'ouverture rend les deux paires ; sa période est libre — quand aucun exercice ne couvre `from`,
`fiscal_year_start_containing` rend `None` et il passe une paire **nulle** « depuis le début d'exercice », ce qui garde
l'ouverture d'un compte de résultat à zéro comme aujourd'hui) ; ses tests restent verts sans retouche d'assertion. ⚠️ `TrialBalanceRow`
dérive `sqlx::FromRow` (`trial_balance.rs:35`) : un champ calculé en Rust ferait échouer la lecture **à l'exécution**
(`ColumnNotFound`), invisible à la compilation — lire dans une structure brute intermédiaire, puis construire la
ligne.

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
d'`account_id`, pas de lien vers le grand livre). ⚠️ La bascule sur « Perte reportée » n'existe **qu'à l'écran**, comme
au bilan (`BalanceSheetView.svelte:97-100`) ; le CSV et le PDF du bilan portent un libellé fixe (`csv.rs:140-144`,
`pdf.rs:596-599`) — la balance fait de même.

**AC 5 — Un contrôle, pas une tautologie.** `TrialBalance` gagne **un** indicateur `opening_balanced` (`bool`) :
`Σ debit_sense(row.opening_balance) − retained_earnings = 0`, où
`debit_sense(v) = if is_debit_natured(row.account_type) { v } else { −v }` — les soldes stockés sont **déjà signés par
type** : les additionner sans re-signer les comptes à nature créditrice donnerait un faux ⚠️ sur des livres exacts
(validation P1, calcul : 1000 − 600 ≠ 0 sur un jeu équilibré). Vérification chiffrée : ventes 1000 et charge 400 en
exercice 1, balance de l'exercice 2 → ouvertures en sens débit 1000 (clients) − 400 (caisse) = 600, `retained_earnings`
600, contrôle nul. **Pas de `closing_balanced`** (validation P2) : Σ debit_sense(clôture) = Σ debit_sense(ouverture) +
(total débit − total crédit des mouvements), et des mouvements déséquilibrés font déjà échouer le rapport
(`TrialBalanceUnbalanced`) — un second indicateur serait la copie du premier, une identité de construction. Le résultat reporté est calculé **indépendamment** (comme au bilan), donc
l'égalité est une vérification réelle, pas une identité de construction. Un déséquilibre n'est **pas** une erreur (à la
différence de `TrialBalanceUnbalanced` sur les mouvements) : il est **journalisé** en `tracing::warn!` — le niveau
d'`equation_holds` au bilan (`balance_sheet.rs:237`) — et affiché (⚠️).

**AC 6 — La règle d'inclusion.** Un compte est rendu s'il est **actif**, **ou** s'il a un mouvement dans la période,
**ou** si son **solde d'ouverture est non nul**. Un compte archivé à solde d'ouverture non nul sans mouvement apparaît
donc (marqueur `active: false`), sinon la clôture ne concorderait pas avec le bilan. Doc du module mise à jour.

**AC 7 — Les rendus.**
- **Écran** (`TrialBalanceView.svelte`) : colonnes N° | Intitulé | **Ouverture** | Débit | Crédit | **Clôture** — clés
  **neuves et courtes** `reports-column-opening` (« Ouverture ») et `reports-column-closing` (« Clôture ») dans les 4
  locales (les clés du grand livre rendent « Solde d'ouverture », trop long et non ancrable) ; la colonne « Solde » (net
  des mouvements) disparaît de l'écran ; pied : totaux débit/crédit et ✓/⚠️ des mouvements comme aujourd'hui, plus
  ✓/⚠️ d'`opening_balanced` sous la colonne Ouverture (AC 5) ; la ligne calculée (AC 4) avant le pied.
  La note `reports-trial-balance-period-note` est **retirée** (clé supprimée des 4 locales) — elle deviendrait fausse.
- **CSV** : `NumeroCompte;NomCompte;SoldeOuverture;TotalDebit;TotalCredit;SoldeCloture` — la colonne `Solde` est
  **remplacée**, pas renommée en silence (CHANGELOG — ⚠️ changement incompatible pour qui lit ce CSV, dans une version
  0.x : **arbitré par Guy le 2026-10-03 : remplacer**). Ligne de résultat reporté sur le patron du bilan
  (`csv.rs:139-145`) : numéro **vide**, libellé fixe « Résultat reporté (calculé) » dans la colonne du nom, ouverture et
  clôture remplies, débit et crédit vides ; puis ligne `Total` (débit, crédit ; ouverture et clôture vides).
- **PDF** : six colonnes en A4 portrait. La mise en page à cinq colonnes occupe **déjà** toute la largeur
  (20 + 70 + 3 × 35 = 195 mm = 210 − 15, validation P1) : positions imposées `MARGIN_LEFT_MM +` **0** (N°), **20**
  (intitulé), **75** (ouverture), **105** (débit), **135** (crédit), **165** (clôture) — colonnes de montant de 30 mm
  (« -1'234'567.89 » y tient), intitulé de 55 mm tronqué à **22** caractères (au lieu de 30, constante nommée).
  En-têtes : deux champs **neufs** de `SectionLabels`, `col_opening` = « Ouverture » et `col_closing` = « Clôture »
  (défauts fr de `PdfContext::fr_ch_default`) — « Solde d'ouverture » en Helvetica-Bold 10 pt mesure 29,85 mm et
  toucherait « Débit », « Solde de clôture » finirait à 2,5 mm du bord (validation P2). Ligne `retained_result_label`. Garde « vide » inchangée
  (`rows.is_empty()`).
- `isReportEmpty('trial-balance', …)` (`reports.api.ts:169-172`) : inchangé — les comptes actifs sont toujours rendus.

**AC 8 — L'API et les types.** `openingBalance`, `closingBalance` sur chaque ligne ; `retainedEarnings`,
`openingBalanced` au niveau du rapport — dans la réponse de `GET /api/v1/reports/trial-balance` et dans
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
- **la frontière** : dans la période mars–juin, une écriture datée **le 1er mars** (comptée dans les mouvements,
  absente de l'ouverture) et une datée **la veille du début de période** (dans l'ouverture, absente des mouvements) —
  la mutation `<` → `<=` compterait la première deux fois, et seules les comparaisons au bilan et au grand livre la
  voient (celle au grand livre seulement si la borne n'est pas dans le module partagé) ;
- une période **d'un seul jour** (`start = end`), une écriture ce jour-là et une la veille ;
- le **premier exercice** d'une société (aucune écriture antérieure) : `retained_earnings = 0`, ouvertures nulles,
  contrôle vrai ; une **société sans aucune écriture** : tout à zéro, contrôle vrai ; un **exercice 2 ouvert et encore
  vide** après un exercice 1 mouvementé — la première vue réelle d'une année neuve : ouvertures = clôtures de
  l'exercice 1 pour les comptes de bilan, nulles pour les comptes de résultat, mouvements nuls, contrôle vrai ;
- des **pertes cumulées** (charges > produits en exercice 1) : `retained_earnings < 0`, contrôle vrai ; le libellé
  « Perte reportée » est vérifié **dans le Vitest de la vue** (seul rendu qui bascule, AC 4) ;
- `opening_balanced` vrai dans ces scénarios — et **faux** quand l'égalité est réellement cassée : une ligne
  d'écriture **déséquilibrée insérée en SQL brut** (hors `create_in_tx`) datée dans l'exercice 1 — sur un compte de
  bilan **et**, dans un second cas, sur un compte de résultat (l'écart passe alors par `retained_earnings`) → balance
  de l'exercice 2 rendue (pas d'erreur : les mouvements de la période restent équilibrés), `opening_balanced` à
  `false`. Sans ce test, le contrôle reproduit l'angle mort d'`equation_holds`, jamais testé à
  `false` dans le dépôt (validation P1) ;
- un test **unitaire** de `debit_sense` (compte à nature créditrice re-signé) ;
- un compte **archivé** à solde d'ouverture non nul, sans mouvement → présent, clôture = bilan ; et un compte de
  **résultat** archivé, mouvementé en janvier, balance de mars–juin → présent (ouverture non nulle), `opening_balanced`
  vrai (une règle d'inclusion limitée aux comptes de bilan le ferait disparaître et le contrôle rougirait) ;
- non-régression : mouvements, `balance`, `balanced` identiques à avant sur le jeu existant (`report_aggregates.rs`) ;
- le test existant `concordance_compte_de_resultat` (`general_ledger.rs:135-205`) étendu à `opening` / `closing` **et
  passé à une période en cours d'exercice** — sur l'exercice entier, les deux ouvertures valent 0 par construction et
  une mutation « ouverture toujours nulle » survivrait.
- Rendus : CSV (en-tête à six colonnes, **valeurs** d'ouverture et de clôture dans l'ordre des colonnes, ligne de
  résultat reporté, ligne `Total`) ; PDF (rendu non vide avec la ligne calculée ; la troncature à 22 caractères testée
  sur un intitulé de 38) ; **Vitest** de la vue (`TrialBalanceView.test.ts`, à créer : colonnes, ligne calculée, ✓/⚠️, note absente) ;
  test d'API sur la forme de la réponse (`reports_e2e.rs`) ; **E2E** : la balance affiche les colonnes Ouverture /
  Clôture (`reports.spec.ts`, sélecteurs ancrés).

Chacun aurait échoué avant le patch ; chaque test de concordance nomme la mutation qu'il tue (ex. « ouverture des
comptes de résultat cumulée depuis l'origine », « règle d'inclusion sans le solde d'ouverture »).

**AC 10 — Le manuel et le CHANGELOG.** `user-manual.tex` § *Balance des comptes* (`:1557-1559`) réécrit : les quatre
colonnes, la règle d'ouverture (bilan depuis l'origine, résultat depuis le début de l'exercice), la ligne calculée,
la concordance avec le bilan et le grand livre ; la phrase sur le **filtre de niveau de détail**, qui n'existe pas,
**retirée**. Le grand livre (`:1561-1565`, « là où la balance dit la même chose autrement ») relu. Le manuel dit aussi : (a) les
soldes de départ sont une écriture datée du premier jour du premier exercice (`:615`) — ils apparaissent donc en
**mouvements** de cet exercice, ouverture nulle ; (b) Ouverture et Clôture sont **signées selon la nature du compte** :
leur somme visible n'est pas nulle (1000 − 400 + 600 = 1200 dans l'exemple de l'AC 5) alors que l'écran affiche ✓ —
le contrôle porte sur les soldes en sens débit. Glossaire `:2043`
corrigé (« à une date donnée » → sur une période, ouverture et clôture). PDF régénéré, contrôlé **aplati**
(`pdftotext … | tr '\n' ' ' | tr -s ' '`). CHANGELOG, section `## [0.12.1] — Non publié` (il n'y a pas de section `[Unreleased]`) : `Changed` (colonnes de l'écran, du PDF et du CSV —
`Solde` remplacé) et `Fixed` (#385). README et site relus (`grep -rn "balance" README.md website/`).

## Tasks / Subtasks

- [x] **T1 — le module partagé** (AC 2) : extraire `is_debit_natured`, `signed`, la borne basse de l'ouverture ; le grand
  livre l'appelle ; ses tests verts sans retouche d'assertion.
- [x] **T2 — la balance** (AC 1, 3, 4, 5, 6) : requête agrégée, champs neufs, ligne calculée, contrôle, inclusion.
- [x] **T3 — les rendus** CSV et PDF (AC 7), fixtures et bench complétés.
- [x] **T4 — l'écran, les types, l'i18n** (AC 7, 8) : vue, `reports.types.ts`, clé de note retirée et deux clés de
  colonne ajoutées dans les 4 locales, `sitesTotal`.
- [x] **T5 — tests** (AC 9).
- [x] **T6 — manuel, CHANGELOG** (AC 10).
- [x] **T7 — gates** : backend complet, frontend complet, **E2E complet**.

## Dev Notes

### Ce qu'il ne faut pas faire

- **Ne pas** recopier `opening_balance` du grand livre dans `trial_balance.rs` (DRY, AC 2) — ni l'appeler compte par
  compte (une requête par compte sur un plan de 300 comptes).
- **Ne pas** changer le sens de `balance` : l'API est publique (clés PAT, Story 15-x). On **ajoute**.
- **Ne pas** fusionner la ligne calculée avec un compte physique de rôle `RetainedEarnings` (collision D1 de la 14-3c,
  `balance_sheet.rs:40-55`) : le compte physique reste une ligne de compte, la ligne calculée reste à part.
- **Ne pas** borner l'ouverture des comptes de bilan par `fiscal_year_id` : c'est exactement le défaut de #385.
- **Ne pas** introduire de filtre de niveau de détail pour rendre le manuel vrai : c'est le manuel qu'on corrige.
- La « requête agrégée unique » (AC 2) n'est tenue par **aucun test** (pas de compteur de requêtes dans le dépôt) :
  c'est un contrôle de **revue de code**, assumé comme tel.

### Limites assumées (à écrire au manuel et en doc de module)

- **Signe lu sur le type courant** du compte : retyper un compte mouvementé re-signe tout son historique — limite
  préexistante du grand livre (#274, #382), héritée telle quelle.
- **Pas d'écriture de clôture** : la remise à zéro des comptes de résultat n'existe que comme borne basse du `SUM` ;
  c'est ce qui rend les tests de concordance (AC 9) indispensables.

### Modules

`kesh-report` (balance, module partagé, rendus, deux libellés PDF), `frontend` (vue, types, test), `kesh-i18n` (clé
retirée, deux clés de colonne), + `docs`, `CHANGELOG`. `kesh-api` : aucun changement de code attendu (la route sérialise la structure ;
les libellés PDF existent) — tests seulement. **Trois modules de code**, sous le seuil de la § *Règle de splitting
préventif*.

### References

- Issue [#385] ; audit du 2026-08-26 (comptable § 5.2, DAF § 1.3).
- `crates/kesh-report/src/general_ledger.rs:1-60`, `:182-275` — la règle et son implémentation.
- `crates/kesh-report/src/balance_sheet.rs:1-120`, `:224-228`, `:333` — le calcul de `retained_earnings`.
- Story 24-1 (grand livre) — `_bmad-output/implementation-artifacts/24-1-grand-livre.md`.

## Dev Agent Record

### Agent Model Used

Claude Opus 5.5 (`claude-opus-5-5`).

### Debug Log References

- Les dix tests de concordance ont passé **du premier coup** : aucun n'a donc été vu rouge avant le code. D'où les
  mutations ci-dessous, qui tiennent lieu de phase rouge.
- Deux garde-fous du frontend ont rougi au gate, et ils avaient raison :
  - `e2e-selecteurs-traduits` : l'E2E ancrait l'en-tête « Clôture » sur son libellé traduit. Remplacé par les
    `data-testid` `tb-col-opening` et `tb-col-closing`, et l'absence de « Solde » par un compte de six en-têtes.
  - `i18n-libelle-en-dur` : la déclaration neuve `retainedLabel` est **nommée** dans le relevé (44 → 45,
    `conforme` 38 → 39). Ses deux branches délèguent à `i18nMsg`.
- Manuel : les glyphes ✓ et ⚠️ manquent dans la police (`Missing character`). Remplacés par des mots. Les dix
  `Missing character` restants sont antérieurs à la story.
- `cargo fmt` a reformaté le fichier de test neuf.

### Completion Notes List

- **T1, le module partagé** — `crate::opening` porte `is_debit_natured`, `is_bilan`, `signed`, `debit_sense` et
  `opening_from`. Le grand livre lit désormais les deux paires de sommes en **une** requête par compte. Quand aucun
  exercice ne couvre `from`, la borne basse vaut `from` lui-même et la paire est nulle. Ses tests passent sans
  retouche d'assertion ; ses deux tests unitaires de signe et de nature ont migré dans `opening.rs`.
- **T2, la balance** — une requête agrégée rend trois paires de sommes brutes par compte, sans aucun `CASE` sur le
  type ni sur le signe. Elle est lue dans une structure brute `RawRow`, la ligne étant construite en Rust.
  - Règle d'inclusion : actif, ou mouvementé, ou porteur d'un solde d'ouverture.
  - `retained_earnings` vient de `balance_sheet::fetch_retained_earnings`, passé en `pub(crate)` et non recopié.
  - `opening_balanced` est calculé par `debit_sense` ; un écart est signalé par `tracing::warn!`.
  - `balance`, `total_debit`, `total_credit` et `balanced` sont inchangés.
  - Les trois tests unitaires tautologiques de `trial_balance.rs` (qui comparaient des constantes) sont remplacés
    par trois tests de `is_opening_balanced`.
- **T3, les rendus** — CSV à six colonnes, avec la ligne du résultat reporté sur le patron du bilan. PDF à six
  colonnes aux positions imposées, avec l'intitulé tronqué à 22 caractères et deux libellés neufs, `col_opening` et
  `col_closing`. Le libellé de la ligne calculée n'est pas tronqué : environ 46 mm pour 55 mm disponibles.
- **T4, l'écran** — les colonnes Ouverture et Clôture, la ligne calculée (« Perte reportée » si elle est négative)
  et le contrôle ✓/⚠️ d'ouverture, distinct de celui des mouvements. La note retirée a perdu sa clé dans les 4
  locales ; deux clés neuves sont ajoutées. `sitesTotal` passe de 1756 à 1758.
- **T5, les tests** — **périmètre : de `HEAD` = `42623926` (fiche validée, aucun code) à l'arbre de travail.**
  - Rust, **+14 nets** :
    - `trial_balance_opening.rs` : 10 tests neufs ;
    - `opening.rs` : 4 tests, dont 2 migrés du grand livre (`general_ledger.rs` passe de 4 à 2) ;
    - `pdf.rs` : +2 ;
    - `trial_balance.rs` : 3 tests remplacés ;
    - `csv.rs` : un test réécrit, qui porte maintenant les valeurs ;
    - `concordance_compte_de_resultat` étendu à une période en cours d'exercice ;
    - `reports_e2e.rs` : un test étendu à la forme de la réponse.
  - Vitest : **+6** dans `TrialBalanceView.test.ts`, neuf. E2E : `reports.spec.ts` étendu.
- **Gates** (T7) : base remise à zéro, `scripts/test-fast.sh` **2545/2545, 4 ignorés** (fmt, clippy, nextest) ;
  frontend `check` 0 erreur, `lint-i18n-ownership` PASS, `test:unit` **845/845**, `build` OK ; **E2E complet** sur
  `kesh_e2e` migrée à neuf (run de 10:07 UTC) : **224 passés, 12 échoués, 19 ignorés** — les 10 attendus d'un run matinal (KF-029 ×7, KF-045 ×2
  `invoices.spec.ts:415/439`, KF-046 `sidebar-navigation:75`) et la paire de pollution documentée `products:166` +
  `product-revenue-account:133`, **vertes rejouées seules** ; `reports.spec.ts` vert, extension 25-5-b comprise.
- ⚠️ **Deux tests PDF prouvent moins que l'AC 9 ne le demande** (revue P1, C-2) : `…_is_not_the_empty_render` compare
  des longueurs, et le test de troncature appelle la fonction et la constante, pas le site d'appel du rendu. Le PDF ne
  s'inspecte pas sans extraire son texte ; limite assumée, aucune mutation PDF n'est déclarée tuée par eux.
- **Mutations tuées** :
  - `<` → `<=` sur la borne d'ouverture : 2 tests tombent ;
  - ouverture de résultat cumulée depuis l'origine : 5 tests ;
  - bilan borné par l'exercice (le défaut de #385) : 5 tests ;
  - ancienne règle d'inclusion : 1 test ;
  - contrôle tautologique : 2 tests ;
  - à l'écran, contrôle lu sur `balanced` : 1 test ;
  - ouverture affichée à la place de la clôture : 1 test.
- **T6, manuel et CHANGELOG** :
  - § *Balance des comptes* réécrite. La phrase sur le filtre de niveau de détail, qui n'existait pas, est retirée.
  - Les deux points de la validation P2 sont écrits : les soldes de départ en mouvements, les montants signés.
  - Le renvoi vers le grand livre est corrigé, ainsi que le glossaire ; le label `sec:soldes-depart` est posé.
  - PDF régénéré et contrôlé aplati.
  - CHANGELOG `[0.12.1]` : une entrée `Fixed` et une entrée `Changed`, cette dernière signalant l'incompatibilité CSV.
  - README et site relus, toujours vrais.

### File List

- `CHANGELOG.md`
- `crates/kesh-api/tests/reports_e2e.rs`
- `crates/kesh-i18n/locales/{de-CH,en-CH,fr-CH,it-CH}/messages.ftl`
- `crates/kesh-report/benches/export.rs`
- `crates/kesh-report/src/{balance_sheet.rs,csv.rs,general_ledger.rs,lib.rs,pdf.rs,trial_balance.rs}`
- `crates/kesh-report/src/opening.rs` (neuf)
- `crates/kesh-report/tests/general_ledger.rs`
- `crates/kesh-report/tests/trial_balance_opening.rs` (neuf)
- `docs/manual/fr/user-manual.{tex,pdf}`
- `frontend/src/lib/features/reports/{TrialBalanceView.svelte,reports.types.ts}`
- `frontend/src/lib/features/reports/TrialBalanceView.test.ts` (neuf)
- `frontend/src/lib/shared/{i18n-keys.test.ts,i18n-libelle-en-dur.test.ts}`
- `frontend/tests/e2e/reports.spec.ts`

## Change Log

- **2026-10-03** — Arbitrage de Guy (« ok ») : la colonne CSV `Solde` est **remplacée**, comme livré.
- **2026-10-03** — Revue de code P2 ciblée (Haiku, prompt `25-5-b-review-prompt-p2.md`) : **0 finding**, sorties des
  quatre vérifications fournies. Le seul résidu cité (« montant au compte de résultat », AC 9) relu par
  l'orchestrateur : il porte sur l'exercice entier, où l'égalité tient. **Boucle close** : P1 1 MED (Sonnet ×3) → P2 0
  (Haiku). Gate complet après la remédiation : base remise à zéro, `scripts/test-fast.sh` **2547/2547, 4 ignorés** ;
  frontend inchangé depuis son gate ; E2E rejoué entre-temps : 227 passés, 9 échoués — les 9 attendus d'un run
  matinal —, 19 ignorés. Statut → `done`.
- **2026-10-03** — Revue de code P1 (Sonnet ×3, prompt `25-5-b-review-prompt-p1.md`) : **0 CRITICAL/HIGH, 1 MED**, LOW.
  - **C-1 (MED)** : le manuel et le CHANGELOG promettaient la concordance d'un compte de **résultat** avec le compte de
    résultat **sur toute période**. C'est faux en cours d'exercice, où le compte de résultat ne porte que les mouvements
    (mars–juin : 820 contre 570). Corrigé : clôture « depuis le début de l'exercice », égale au compte de résultat
    quand la période commence au premier jour de l'exercice, au grand livre toujours. PDF régénéré, contrôlé aplati.
  - **LOW corrigés** :
    - le contrôle d'ouverture n'existait qu'à l'écran ; le PDF l'écrit désormais sous les totaux quand il tombe (A-1,
      nouveau libellé `opening_unbalanced`, test, mutation tuée) ;
    - un compte archivé aux mouvements compensés n'était exercé par aucun test (C-3, test neuf, mutation « inclusion
      sur le montant net » tuée) ;
    - les ignorés manquaient aux gates déclarés (C-4) ;
    - la limite des deux tests PDF est écrite (C-2) ;
    - deux limites sont écrites en doc de module : l'invariant date ↔ exercice, tenu par `create_in_tx`
      (`DateOutsideFiscalYear`), vérifié (A-4, B-2) ; et la lecture de tout l'historique (A-5, B-1).
  - **Écartés** :
    - « Perte reportée » réservé à l'écran, voulu par l'AC 4 (A-2) ;
    - la convention de signe, écrite au manuel (A-3) ;
    - la ligne du résultat reporté affichée à 0, voulue (B-3).
  - Gate ciblé : `fmt`, `clippy kesh-report`, `binary(trial_balance_opening)` + tests `trial_balance` 21/21. **Gate complet
    au push.**
- **2026-10-03** — Implémentée (dev-story) : T1–T7, gates ci-dessus. Statut → `review`.
- **2026-10-03** — Validation P3 ciblée (Haiku, prompt `25-5-b-validate-prompt-p3.md`) : **0 finding**, sorties des quatre
  vérifications fournies. Le point 2 (compatibilité de l'AC 2 avec le grand livre), établi par raisonnement, **repris
  par l'orchestrateur** : la période libre du grand livre (`LedgerPeriod`) impose de passer une paire nulle quand aucun
  exercice ne couvre `from` — écrit à l'AC 2 (LOW). **Boucle close** : P1 1H/4M (Sonnet ×3) → P2 4M/9L (Opus) → P3 0
  (Haiku). ⚠️ Arbitrage de Guy attendu sur le CSV (remplacer `Solde` ou ajouter deux colonnes en fin de ligne) — défaut
  retenu : remplacer.
- **2026-10-03** — Validation P2 (Opus, prompt `25-5-b-validate-prompt-p2.md`) : **0 CRITICAL/HIGH, 4 MED, 9 LOW**,
  retenus. `closing_balanced` retiré — copie d'`opening_balanced` par construction (M1) ; la forme de l'AC 2 prescrite
  (sommes brutes en SQL, règle en Rust partagée avec le grand livre ; piège `FromRow`) (M2) ; libellés courts
  « Ouverture » / « Clôture » — les en-têtes du grand livre débordent au PDF (calcul AFM) et ne s'ancrent pas en E2E
  (M3) ; « Perte reportée » à l'écran seulement, comme au bilan (M4) ; test de concordance du grand livre en cours
  d'exercice, « société sans écriture » et exercice 2 vide, compte de résultat archivé, `warn!`, CHANGELOG `[0.12.1]`
  et incompatibilité CSV à arbitrer, « la veille », colonne du libellé CSV, deux phrases au manuel (L1–L8). Trend :
  P1 1H/4M → P2 4M.
- **2026-10-03** — Validation P1 (Sonnet ×3, prompt `25-5-b-validate-prompt-p1.md`) : **1 HIGH, 4 MED**, LOW. Retenus :
  la frontière `entry_date = start` testée, et la veille (C1, HIGH) ; la formule du sens débit écrite et chiffrée (A) ;
  le contrôle testé à `false` par une ligne déséquilibrée insérée en SQL brut (C2) ; premier exercice (C3) ; positions
  des six colonnes du PDF chiffrées — la largeur est déjà saturée à cinq (B) ; période d'un jour, exercice vide, pertes
  cumulées, valeurs CSV, troncature, requête unique laissée à la revue, deux références décalées (LOW). Règle, ligne
  calculée et contrôle **confirmés par le calcul** sur trois jeux (lentille A, mutation #385 détectée : −1000 ≠ 0).
- **2026-10-03** — Créée (Guy : « pousse et continue »).

[#385]: https://github.com/guycorbaz/kesh/issues/385
