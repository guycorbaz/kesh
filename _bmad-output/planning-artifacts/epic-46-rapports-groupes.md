# Epic 46 — Rapports par groupes de comptes

**Kickoff : 2026-11-18** (branche `epic/46-rapports-groupes`, partie de `main` à `773df80d`, release v0.19.0).
Numéro **46** réservé par Guy ; responsable de l'epic (contributeur). État : **revue 1 approuvée par le responsable de l'epic et par Guy
(2026-11-19)** ; `main` reprise le 2026-11-20 (`d29aa557` : #1346, #1327, #1351 fusionnées) ; **fiches écrites le
2026-11-20, `drafted`**, validation en boucle à suivre (`CLAUDE.md` § *Review Iteration Rule*). Aucun code écrit.

**Cible** : la release qui suit la fusion de sa PR, numéro attribué au tag (un epic = une release) ; l'epic part
de `main`, qui porte déjà la 48-11 (`kesh-report/src/pdf.rs`, remarque 1 de Guy) *(« 0.21.0 (prévision) » jusqu'au
2026-11-25, retiré à la reprise sur `main` : on parle d'epics, le numéro n'est acquis qu'au tag)* ; CHANGELOG daté,
macros des manuels et tag par Guy. **Aucune migration attendue** (D2).

**Taille — epic de fonctionnalité nouvelle, au-delà de la borne de six, annoncé ici** (`CLAUDE.md` § *Numérotation
des versions*) : **sept stories** à la revue 1 (46-1 à 46-7, solde compris), approuvées par Guy ; la balance par
groupes (« 46-6 ») touche plus de cinq modules et se découpe, par la règle de splitting préventif, en **46-6a**
(modèle, API, MCP) et **46-6b** (écran, PDF, CSV) — **huit fiches**, la balance restant au numéro 46-6 et le solde
en dernier (remarque 2 de Guy). 46-5 était **conditionnelle** (Q9, maquette avant décision) : **décidée le 2026-11-22** par le
responsable de l'epic sur la maquette, découpée en **46-5a** et **46-5b** (six modules), et fusionnée dans l'epic
seulement après l'accord de Guy ; s'il la refuse, elle sort de l'epic. **Accord du mainteneur le 2026-11-26** : la 46-5
entre dans la PR de l'Epic 46 (branche `pr/46-rapports-groupes-v2`), pour que l'epic n'entre qu'une fois dans `main`.
*(Annoté le 2026-11-26.)*

**Règles des fichiers communs** (`CLAUDE.md` § *Travailler à plusieurs développeurs*) : aucune ligne au `CHANGELOG.md`
en cours d'epic (chaque fiche a sa *Note de version*) ; aucun PDF de manuel commité (`.tex` seul) ; catalogues i18n et
`sprint-status.yaml` en bloc propre ; `sitesTotal` et autres compteurs recomptés après chaque reprise de `main`.

## Intention

Le bilan et le compte de résultat se présentent **par groupes de comptes**, avec des **sous-totaux tirés de la
hiérarchie du plan** (`accounts.parent_id`), et un **niveau de détail choisi au moment de l'affichage ou de
l'impression** : des seuls groupes de premier niveau jusqu'au détail de chaque compte. Écran, PDF, CSV, API et MCP.

Besoin réel du responsable de l'epic : son ancien logiciel présentait le bilan par groupes (1 ACTIFS › 10 Actifs circulants ›
100 Trésorerie › comptes, avec le solde en devise d'un groupe). Il structure son plan ainsi : **10 numéros réservés
par banque** (néobanques comprises : Wise, PayPal…), un compte par devise, **un compte de groupe par banque = le numéro
sans son dernier chiffre**, puis un groupe de toutes les liquidités, plus court encore (**groupe 10**).

**Hors périmètre** (prévu plus tard) : des **modèles de sortie** (presets) qui enregistrent ces choix ; les rubriques
légales imposées (Q7) ; le grand livre (Q6) ; **la balance est dans le périmètre** (46-6, responsable).

## Inventaire du code (relevé le 2026-11-18 à `773df80d`)

| Domaine | Ce qui existe | Où |
|---|---|---|
| Hiérarchie | `accounts.parent_id` (FK `ON DELETE RESTRICT`), `postable` (un parent devient non imputable, 14-3a D2) ; **aucune contrainte de type** entre parent et enfant | `migrations/20260411000001_accounts.sql:12,18` ; `kesh-db/src/entities/account.rs:228-240` |
| | parent modifiable par `PUT /accounts/{id}` : **non fusionné** (`fix/api-parent-compte`, `review`) ; refuse un cycle et un parent qui porte des écritures | fiche `fix-api-parent-compte.md` AC-2, AC-3 |
| Plan PME livré | 86 comptes, trois niveaux : classe (`1` Actifs) › groupe (`10` Actifs circulants) › comptes (`1000`…) ; 9 classes | `kesh-core/assets/charts/pme.json` |
| Bilan | sections par **type** et **rôle** : `assets`, `liabilities` (dettes), `equity` (rôles de fonds propres, ordre CO 959a), lignes calculées `retained_earnings`, `equity_result` ; liste **plate** triée par numéro ; `HAVING balance != 0 OR balance_currency != 0` | `kesh-report/src/balance_sheet.rs:74-330` |
| Compte de résultat | `revenues`, `expenses`, plats, même `AccountBalance` | `kesh-report/src/income_statement.rs` |
| Devise | `AccountBalance.currency`, `balance_currency` (37-4) ; aucun total en devise (NFR-37-4-3) | `balance_sheet.rs:152-180` |
| Rendus | PDF `render_balance_sheet_pdf`, `render_income_statement_pdf` (sous-titres de rôle) ; CSV `render_balance_sheet_csv`, `render_income_statement_csv` | `pdf.rs:636,766` ; `csv.rs:142,271` |
| API | `GET /api/v1/reports/{balance-sheet,income-statement}[/export]`, `ReportQuery`, `ExportQuery` | `kesh-api/src/lib.rs:1381-1430` ; `routes/reports.rs:57-200` |
| MCP | `kesh_bilan`, `kesh_compte_de_resultat` (`ArgumentsPeriode`, `deny_unknown_fields`) | `kesh-api/src/mcp/outils/rapports.rs` |
| Écrans | `BalanceSheetView.svelte` (groupes de rôle), `IncomeStatementView.svelte`, `reports.api.ts`, `reports.types.ts` | `frontend/src/lib/features/reports/` |
| Lecture de `parent_id` | **aucun rapport** ne la lit (relevé de la fiche `fix-api-parent-compte`, AC-7) | — |

## Décisions (proposées au kickoff, approuvées par le responsable de l'epic et par Guy le 2026-11-19 ; annotations du responsable de l'epic sous chacune)

- **D1 — Les groupes viennent de `parent_id` seul**, jamais du numéro (comme la règle « jamais par numéro » de 14-1).
  Un groupe = tout compte qui a au moins un sous-compte. Le préfixe de numéro ne sert qu'à l'outil d'aide (46-5). *(Q1)*
  *Responsable de l'epic, 2026-11-19 :* approuvé (Q1 : par `parent_id`).
- **D2 — Aucune migration.** Les soldes par compte restent calculés par les requêtes actuelles ; une seule lecture
  de l'arbre des comptes de la société s'y ajoute ; l'agrégation est une **fonction pure** (`kesh-report/src/groupes.rs`),
  testée sans base.
  *Responsable de l'epic, 2026-11-19 :* approuvé.
- **D3 — Niveau de détail = profondeur dans l'arbre**, non longueur du numéro : niveau 1 = racines, niveau k =
  nœuds de profondeur ≤ k, « tous » = chaque compte. Un compte plus profond que le niveau est **replié** dans son
  ancêtre de ce niveau. Un compte sans parent (racine sans enfant) s'affiche à tout niveau. Le mode **« à plat »**
  (sans groupes) reste celui d'aujourd'hui.
  *Responsable de l'epic, 2026-11-19 :* approuvé.
- **D4 — Le squelette légal du bilan demeure** : Actifs, Dettes, Fonds propres, puis les deux lignes calculées (14-1,
  14-3c, CO 959a). Les groupes s'emboîtent **dans** chaque section. Un groupe dont les comptes tombent dans plusieurs
  sections (p. ex. `2 Passifs` = dettes + fonds propres ; `7 Résultats annexes` = produits + charges) **n'est pas
  affiché** : ses enfants remontent d'un niveau. Les totaux restent exacts. Les lignes calculées restent hors groupe,
  en fin de fonds propres. En mode groupé, l'ordre des fonds propres suit l'arbre (numéros) ; sous-titres de rôle
  seulement en mode à plat. *(Q4)*
  *Responsable de l'epic, 2026-11-19 :* approuvé (Q4 : squelette légal gardé, groupe à cheval dissous).
- **D5 — Solde propre d'un groupe.** Un compte de groupe qui porte lui-même des écritures (admis à la création) donne
  une ligne « solde propre » parmi ses enfants ; son sous-total l'inclut.
  *Responsable de l'epic, 2026-11-19 :* approuvé.
- **D6 — Groupes vides masqués** : un groupe dont le sous-total CHF **et** le solde en devise sont nuls ne s'affiche
  pas (même règle que `HAVING` des comptes). *(Q2)*
  *Responsable de l'epic, 2026-11-19 :* **modifié** (Q2) : un groupe non affiché laisse son total à son parent ; pas d'option. Les groupes à solde nul restent masqués.
- **D7 — Solde en devise d'un groupe** : donné seulement si **tous** ses comptes à solde non nul sont dans la **même
  devise étrangère** ; sinon absent. Les totaux de section restent en CHF (NFR-37-4-3). *(Q3)*
  *Responsable de l'epic, 2026-11-19 :* **modifié** (Q3) : une seule devise : devise + CHF ; sinon CHF seul, **sans ligne supplémentaire**.
- **D8 — Contrat additif.** Sans paramètre, JSON, CSV et PDF restent **identiques** à aujourd'hui. Avec le paramètre
  `detailLevel` (`1`…`n` | `all`), chaque section gagne un champ `groupedRows` : lignes déjà ordonnées
  `{ kind: group|account|own, depth, accountId, number, name, balance, currency?, balanceCurrency? }`. Les trois rendus
  et le MCP lisent la même liste. *(Q5 pour le MCP)*
  *Responsable de l'epic, 2026-11-19 :* approuvé (Q5 tel que proposé) ; le MCP suit, avec la position des sous-totaux (46-4). *Précisé à la 46-1, § Le contrat (validation P3, 2026-11-20) : les noms de champs de la fiche font foi (`accountNumber`, `accountName`, `open`, `active`, `role`, `mirrorOf`, genre `subtotal` en pied) ; le schéma ci-dessus n'en est qu'une esquisse.*
  *Annoté le 2026-11-24, story 46-8 : « sans paramètre … identiques » ne vaut plus pour l'**export** par l'API, groupé par défaut ; l'export d'avant se demande par `detailLevel=flat`. Voir l'amendement de NFR-46-1, ci-dessous.*
- **D9 — Choix du niveau à l'écran** : un sélecteur « À plat · Niveau 1 · … · Tous les comptes » sur le bilan et le
  compte de résultat ; le niveau passe à l'export (PDF, CSV). Retenu par visiteur (`localStorage`, convenance) ; aucun
  preset enregistré (hors périmètre).
  *Responsable de l'epic, 2026-11-19 :* approuvé, **étendu** (Q10) : hiérarchie visible, accordéons ouverts par défaut ; contrôles « tout ouvrir / tout fermer » et « hiérarchie masquée / montrée » ; **choix de la position des sous-totaux** (sur la ligne du titre au-dessus des comptes, ou ligne de sous-total au-dessous) ; PDF **fidèle à l'écran** ; tout pilotable par l'API et le MCP.
- **D10 — Ordre des frères** : par numéro, comme aujourd'hui (comparaison de chaînes, `ORDER BY a.number`).
  *Responsable de l'epic, 2026-11-19 :* approuvé.
- **D11 — Balance et grand livre hors périmètre.** *(Q6)*
  *Responsable de l'epic, 2026-11-19 :* **modifié** (Q6) : la **balance** passe dans le périmètre (46-6) ; le grand livre reste hors périmètre (« pas utile »).
- **D12 — Structuration du plan par l'utilisateur** : avec `fix/api-parent-compte`, l'écran du plan suffit. L'outil
  d'aide (46-5) **propose**, l'utilisateur confirme ; rien n'est créé d'office (`CLAUDE.md` § *Un appariement
  automatique propose*). *(Q9)*
  *Responsable de l'epic, 2026-11-19 :* approuvé ; maquette d'interface avant de décider 46-5 (Q9).

## Exigences

**Fonctionnelles** (détail par fiche)

- FR-46-1 — Arbre des groupes et sous-totaux (D1, D3–D7, D10), exposé par l'API (D8). → 46-1
- FR-46-2 — Écrans du bilan et du compte de résultat au niveau choisi (D9). → 46-2
- FR-46-3 — PDF et CSV au niveau choisi, groupes en gras, retrait par profondeur. → 46-3
- FR-46-4 — Outils MCP `kesh_bilan`, `kesh_compte_de_resultat` avec le niveau. → 46-4
- FR-46-5 — Aide à la création des groupes d'un plan existant. → 46-5a, 46-5b (décidée le 2026-11-22 sur la maquette ; accord du mainteneur le 2026-11-26)
- FR-46-6 — Balance par groupes (Q6, responsable) : niveaux, sous-totaux, écran, PDF, CSV, API, MCP. → 46-6a, 46-6b
- FR-46-7 — Un groupe non affiché laisse son total à son parent (Q2). → 46-1, 46-2
- FR-46-8 — Solde en devise d'un groupe : devise + CHF si une seule devise, sinon CHF seul, aucune ligne supplémentaire (Q3). → 46-1, 46-2, 46-3
- FR-46-9 — Position des sous-totaux au choix : sur la ligne du titre, au-dessus des comptes, ou en ligne de sous-total au-dessous ; PDF fidèle à l'écran. → 46-2, 46-3
- FR-46-10 — Hiérarchie visible, accordéons ouverts par défaut ; « tout ouvrir / tout fermer » ; « hiérarchie masquée / montrée » (Q10). → 46-2, 46-6b
- FR-46-11 — Tout ce que l'écran et le PDF permettent se pilote par l'API et le MCP. → 46-1, 46-3, 46-4, 46-6a, 46-6b

**Non fonctionnelles**

- NFR-46-1 — Sans paramètre : JSON, CSV (octet pour octet) et PDF (mêmes lignes de texte) identiques à l'actuel.
  *(Amendé le 2026-11-24, décision du mainteneur, story 46-8 : l'**export** par l'API rend la présentation de l'écran
  sans paramètre (groupé, `detailLevel=all`) ; l'export d'avant se demande par `detailLevel=flat`, CSV à l'octet, PDF
  aux mêmes lignes. NFR-46-1 et D8 valent désormais pour le JSON et le MCP, et pour l'export à `detailLevel=flat`.)*
- NFR-46-2 — Somme des sous-totaux de premier niveau = total de la section, à tout niveau ; équation du bilan inchangée.
- NFR-46-3 — Une seule requête de plus par rapport (l'arbre) ; agrégation en mémoire, O(n comptes).
- NFR-46-4 — Arbre défensif : un cycle ou un parent d'une autre société (impossible par l'API, possible par SQL) ne
  fait ni boucle ni panique : le compte est traité en racine, avec `tracing::warn!`.
- NFR-46-5 — Quatre locales ; `i18n-keys`, `i18n-libelle-en-dur` verts ; manuel utilisateur FR (`.tex`).
- NFR-46-6 — MCP : `mcp_contrat`, `mcp_gardes_lexicales` verts, tailles re-mesurées.
- NFR-46-7 — Respect de l'art. 959a CO (Q7, responsable), lecture (a) : la présentation par groupes conserve les sections
  légales du bilan (Actifs, Dettes, Fonds propres, lignes calculées) ; aucun rattachement de compte à une rubrique. → 46-1

## Remarques de Guy sur la revue 1 (2026-11-19) et décisions (2026-11-20)

Guy a approuvé la revue 1 sans condition ; son agent a fait cinq remarques, traitées ainsi à l'écriture des fiches :

1. **Release** : visée **0.21.0** (prévision) ; aucune collision avec la 48-11 (0.20.0), dont l'epic part une fois
   fusionnée.
2. **Balance** : décrite en entier, au numéro 46-6 (découpée en 46-6a et 46-6b, règle de splitting préventif) ; le
   solde reste en dernier (46-7).
3. **Art. 959a CO** : lecture (a) (cf. Q7) ; aucune migration.
4. **Niveau et accordéons** : le niveau fixe l'état de départ ; les ouvertures et fermetures manuelles priment ensuite
   (listes `open` / `closed` de l'API) ; le PDF « fidèle à l'écran » imprime le niveau et les groupes ouverts ou
   fermés, passés à l'API ; critère exprès : 46-1 AC-3, 46-2 AC-3, 46-3 AC-3 et AC-4.
5. **Sous-totaux** : par défaut sur la ligne du titre (`header`) ; sans paramètre, la réponse de l'API reste celle
   d'aujourd'hui (46-1 AC-8, 46-3 AC-2, 46-4 AC-2, 46-6a AC-5).

Contraintes de Guy reprises dans les fiches : la **48-11** met `round_for_display` et `format_two_decimals` dans
`kesh-core` (noms annoncés ; au 2026-11-20 la fiche 48-11 est « à écrire » et aucune branche ne la porte : noms relevés
au T0 de la 46-3) et une garde interdit tout `{:.2}` neuf dans `kesh-report` (46-1 AC-13, 46-3 AC-6, 46-6a AC-8, 46-6b
AC-4 ; la 46-3 ne commence qu'après sa fusion) ; la **47-3** (présentation miroir) s'applique **avant** les groupes
(46-1 AC-9).

## Stories

| Clé | Titre | Dépend de | Taille | Fiche |
|---|---|---|---|---|
| 46-0 | Kickoff (ce fichier ; pas de fiche) | — | — | — |
| 46-1 | Arbre des groupes : modèle et API (le contrat) | `main` reprise (fait) | M | `46-1-arbre-des-groupes-modele-api.md` |
| 46-2 | Écrans du bilan et du compte de résultat par groupes | 46-1, **46-3** (export groupé) | M | `46-2-groupes-ecrans.md` |
| 46-3 | PDF et CSV par groupes | 46-1, **48-11 dans `main`** | M | `46-3-groupes-pdf-csv.md` |
| 46-4 | Outils MCP par groupes | 46-1 | S | `46-4-groupes-mcp.md` |
| 46-5 | Aide à la création des groupes d'un plan — décidée le 2026-11-22 sur la maquette, découpée (six modules) ; accord du mainteneur le 2026-11-26, dans la PR de l'epic | maquette approuvée | — | `46-5-aide-groupes-plan.md` |
| 46-5a | Groupes du plan : transaction et route | — | M | `46-5a-aide-groupes-transaction-route.md` |
| 46-5b | Groupes du plan : propositions, écran, manuel | 46-5a | M | `46-5b-aide-groupes-ecran.md` |
| 46-6a | Balance par groupes : modèle, API, MCP | 46-1 ; MCP : 46-4 | M | `46-6a-balance-groupes-modele-api-mcp.md` |
| 46-6b | Balance par groupes : écran, PDF, CSV | 46-6a, 46-2, 46-3 | M | `46-6b-balance-groupes-ecran-pdf-csv.md` |
| 46-7 | Solde de la dette de l'epic (plafond : **2 issues**) | toutes | S | `46-7-solde-dette-epic.md` |
| 46-8 | L'export par l'API rend la présentation de l'écran (décision du mainteneur du 2026-11-24) | 46-3, 46-6b | S | `46-8-export-api-groupe-par-defaut.md` |

Ordre : 46-1 → (46-3 dès la 48-11 fusionnée, 46-4, 46-6a hors MCP en parallèle ; le MCP de la 46-6a après la 46-4) → 46-2 (après 46-3, pour l'export groupé :
avant elle, `ExportQuery` ignorerait les paramètres et rendrait un PDF à plat d'un écran groupé, sans rien signaler) →
46-6b → 46-7 → 46-5a → 46-5b (décidée après la clôture de la 46-7 : sa dette se corrige dans la story ou s'ouvre en issue, fiche mère). **Ordre avec la 47-3** : miroir puis groupes ; le second arrivé recâble (fiche
47-3, § *Recâblage*), avec revue du code d'intégration.

## Dépendances

- **`fix/api-parent-compte`** : **fusionné** (#1351, `d29aa557`). ⚠️ Il refuse un parent qui porte des écritures :
  un compte déjà mouvementé (p. ex. `1020 Banque`) ne peut pas devenir le groupe d'une banque ; le groupe doit être un
  compte neuf.
- **`fix/bilan-resultat-reporte`** : **fusionné** (#1346, `b5333177`).
- **`fix/pdf-montants-alignes`** : **fusionné** (#1327, `0867f1fb`).
- **48-11 (arrondi au centime, Epic 48, release 0.20.0)** : déplace `round_for_display` et `format_two_decimals` dans
  `kesh-core`, interdit tout `{:.2}` neuf dans `kesh-report`. **46-3 et 46-6b ne commencent qu'après sa fusion.**
  *(Amendé le 2026-11-22, décision du responsable de l'epic : la 46-3 n'attend plus. Elle pose dans `kesh-core` un
  remplaçant minimal, `kesh_core::types::format_two_decimals` — arrondi au centime, l'équidistant loin de zéro, puis
  formatage —, employé par ses seuls rendus groupés ; **provisoire, à remplacer par la 48-11 à l'intégration**.
  Fiche 46-3, § « Amendement du 22.11 : remplaçant de la 48-11 ».)*
- **Epic 47 (comptes miroirs)**, 47-3 (`story/47-3`) : présentation miroir dans le service, générateurs bruts ;
  **miroir puis groupes** (décision de Guy) ; 46 et 47 se croisent dans `services/rapports.rs` et
  `reports_e2e.rs`.

## Fichiers communs

Déclarés au kickoff (`CLAUDE.md` § *Travailler à plusieurs développeurs*, § *Fusions et fichiers communs*), relevés le
2026-11-20 à `29b9ec27` contre la 0.20.0 en cours (Epics 48, 49) et l'Epic 47.

| Fichier ou module | Stories de l'epic | Autres chantiers qui le touchent | Règle |
|---|---|---|---|
| `crates/kesh-report/src/groupes.rs` (neuf, enveloppe `AvecGroupes<T, S>`), `lib.rs` | 46-1, 46-6a | — | — |
| `crates/kesh-report/src/balance_sheet.rs`, `income_statement.rs` | **aucune** (validation P1 de la 46-1, C-46-1-4 : enveloppe, structures et littéraux inchangés) | 47-3 (`mirror_of` d'`AccountBalance`) | — |
| `crates/kesh-report/src/trial_balance.rs` | **aucune** (46-6a : `TrialBalance` inchangé, code de la balance groupée dans `groupes.rs` ; validation P1 de la nouvelle boucle, F12) | — | — |
| `crates/kesh-report/src/pdf.rs` | 46-3, 46-6b | **48-11** (fort : formateurs, garde `{:.2}`), 47-3 (littéraux de tests), 49-1b (PDF des factures, autre région) | 46-3 part de `main` **après** la 48-11 |
| `crates/kesh-report/src/csv.rs` | 46-3, 46-6b | **48-11**, 47-3 (littéraux) | idem |
| `crates/kesh-report/benches/export.rs` | aucune (C-46-1-4) | 47-3 | — |
| `crates/kesh-api/src/services/rapports.rs` | 46-1, 46-6a | **47-3** (`bilan`, `compte_de_resultat` : présentation miroir) | miroir puis groupes (46-1 AC-9) ; recâblage par le second arrivé |
| `crates/kesh-api/src/routes/reports.rs` | 46-1, 46-3, 46-6a, 46-6b | — | — |
| `crates/kesh-api/tests/reports_e2e.rs`, `reports_export_e2e.rs` | 46-1, 46-3, 46-6a, 46-6b | 47-3 (exemple chiffré) | blocs de tests distincts |
| `crates/kesh-api/src/mcp/outils/rapports.rs`, `tests/mcp_*.rs` | 46-4, 46-6a | 47-3 (descriptions de `kesh_bilan`, `kesh_compte_de_resultat`) | garder les deux phrases (aucune garde ne contrôle la longueur des descriptions ; la taille des réponses est mesurée à la 46-4, AC-5) |
| `crates/kesh-api/src/routes/accounts.rs`, `kesh-db/src/repositories/accounts.rs`, `kesh-api/src/lib.rs` (`comptable_routes`), `kesh-api/src/errors.rs`, `kesh-db/src/errors.rs`, `kesh-api/tests/audit_route_registry.rs` (liste (vi) et son décompte, **recompté** après fusion), `docs/MULTI-TENANT-SCOPING-PATTERNS.md` (tableau des verrous) | 46-5a | 47-1/47-2 (miroir), 48-x | blocs distincts |
| `crates/kesh-i18n/locales/*/messages.ftl` | 46-2, 46-6b, 46-5a, 46-5b | tous | bloc propre `reports-groups-*` (46-5 : `accounts-groups-*`) ; conflit : garder les deux côtés |
| `frontend/src/lib/features/reports/*` (vues, `reports.api.ts`, `reports.types.ts`, `GroupedReportRows.svelte` neuf) | 46-2, 46-6b | 47-4 (écrans du miroir) | le second arrivé résout |
| `frontend/src/routes/(app)/reports/+page.svelte` | 46-2, 46-6b | 47-4 | idem |
| `frontend/src/routes/(app)/accounts/+page.svelte`, `features/accounts/` | 46-5b | 47-2 (écran du plan) | idem |
| `frontend/src/lib/shared/i18n-keys.test.ts` (`sitesTotal`) | 46-2, 46-6b, 46-5a, 46-5b | tous | **recompté** par la garde après fusion |
| `frontend/tests/e2e/reports*.spec.ts` | 46-2, 46-6b | 47-4 | 46-2 change l'écran par défaut (Q10) : specs adaptées sans perdre d'assertion |
| `docs/api-external.md` | 46-1, 46-3, 46-6a, 46-5a | 47-3, Epic 48, 49 | paragraphes distincts |
| `docs/manual/fr/user-manual.tex` | 46-2, 46-3, 46-6b, 46-5b | 47-4, 49-1c, v0200-menu-sous-groupes | `.tex` seul, PDF régénérés à la release |
| `docs/manual/fr/admin-manual.tex` (tableau MCP ; opérations fermées aux clés : 46-5a) | 46-4, 46-6a, 46-5a | 47-3 | troisième colonne : garder les deux phrases |
| `_bmad-output/implementation-artifacts/sprint-status.yaml` | toutes | tous | bloc propre de l'epic |
| `CHANGELOG.md` | — | — | jamais en cours d'epic (*Note de version* de chaque fiche) |

## Migrations

Aucune attendue (D2). Si une story en découvre le besoin, elle le dit à sa validation, et l'epic passe par la
sauvegarde d'avant mise à jour (§ *Migration breaking policy*).

## Décomptes

Issues ouvertes et bugs : **à mesurer au kickoff** (au tag v0.19.0 et ce jour, commandes de § *Priorités des
défauts*) ; objectif au tag de la release de l'epic : moins qu'au tag v0.19.0 (objectif, non barrière).

## Questions ouvertes

- **Q1 — Source des groupes** : `parent_id` seul (D1), ou aussi un regroupement par préfixe de numéro sans compte de
  groupe ? *(responsable)*
  *Responsable de l'epic, 2026-11-19 :* par `parent_id`.
- **Q2 — Groupes vides** : masqués (D6), ou option « afficher les groupes vides » ? *(responsable)*
  *Responsable de l'epic, 2026-11-19 :* quand un groupe n'est pas affiché, son parent affiche le total (pas d'option).
- **Q3 — Solde en devise d'un groupe** : une seule devise seulement (D7), ou une ligne par devise (USD x, EUR y) ? *(responsable)*
  *Responsable de l'epic, 2026-11-19 :* si une seule devise : devise + CHF ; sinon CHF seul, pas de lignes supplémentaires.
- **Q4 — Bilan** : squelette légal conservé et groupes à cheval dissous (D4), ou l'arbre remplace les sections
  (`1 ACTIFS`, `2 PASSIFS` comme titres) ? Où placer les lignes calculées ? *(responsable, puis Guy)*
  *Responsable de l'epic, 2026-11-19 :* on garde le squelette légal ; un groupe à cheval est dissous.
- **Q5 — API et MCP** : nom et forme du paramètre (`detailLevel`), champ `groupedRows` ; le MCP en a-t-il besoin ? *(responsable, Guy)*
  *Responsable de l'epic, 2026-11-19 :* approuvé tel que proposé.
- **Q6 — Balance et grand livre** : par groupes aussi (sous-totaux de la balance) ? Proposé : non, plus tard. *(responsable)*
  *Responsable de l'epic, 2026-11-19 :* balance : **maintenant** (story 46-6) ; grand livre : pas utile.
- **Q7 — Rubriques légales du bilan suisse (art. 959a CO)** : imposer ou contrôler la correspondance des groupes aux
  rubriques minimales ? Proposé : non (le plan PME y correspond déjà). *(Guy)*
  *Responsable de l'epic, 2026-11-19 :* respecter l'art. 959a CO (NFR-46-7).
  *Décision du 2026-11-20 (remarque 3 de Guy) :* **lecture (a)** — le bilan garde ses sections légales et les groupes
  s'y rangent (D4) ; aucune donnée neuve, aucune migration ; une présentation repliée (niveau 1 en particulier) est
  **un affichage de travail, non des comptes annuels** (écrit à l'API, 46-1, et au manuel, 46-2 et 46-3).
- **Q8 — Ordre avec l'Epic 47** (comptes miroirs, qui s'appuiera sur ces générateurs) : 46 d'abord ? *(Guy, responsable)*
  *Responsable de l'epic, 2026-11-19 :* approuvé tel que proposé : 46 avant 47 (déjà noté par Guy).
- **Q9 — Outil d'aide (46-5)** : utile, ou l'écran du plan suffit-il après `fix/api-parent-compte` ? Règle de
  proposition : « numéro sans son dernier chiffre » ? *(responsable)*
  *Responsable de l'epic, 2026-11-19 :* l'outil peut être intéressant ; **proposition d'UI (maquette) avant de décider** ; règle « numéro sans le dernier chiffre » à confirmer alors.
- **Q10 — Niveau par défaut à l'écran** : « Tous les comptes » groupé, ou « à plat » comme aujourd'hui ? *(responsable)*
  *Responsable de l'epic, 2026-11-19 :* hiérarchie visible avec accordéons ouverts par défaut ; contrôles « tout ouvrir / tout fermer » et « hiérarchie masquée / montrée » (FR-46-10).

## Revue du responsable de l'epic (2026-11-19)

Sections approuvées (intention, décisions, questions, stories) ; réponses point par point annotées plus haut.
Changements : D6 (total d'un groupe non affiché remonté au parent), D7 (devise + CHF ou CHF seul), D9 (hiérarchie
visible, accordéons, contrôles, position des sous-totaux, PDF fidèle, pilotage par API et MCP), D11 (**balance dans le
périmètre**), Q7 (art. 959a CO respecté), Q9 (maquette avant 46-5).
**Effet sur les stories** : une story de plus, **46-6 Balance par groupes** ; le solde passe en **46-7** ; l'epic compte
sept stories (borne de six dépassée, à confirmer par Guy). 46-2 et 46-3 gagnent le choix de la position des sous-totaux
et la fidélité écran / PDF ; 46-4 porte la position et la balance ; 46-5 attend sa maquette.
La page pour Guy (`data-not-committed/epic-46-revue-pour-guy.html`) porte ces réponses ; **approbation de Guy attendue**.

## Avant le développement

1. ~~Revue du responsable de l'epic et approbation de Guy~~ : faites (2026-11-19).
2. Décomptes d'issues mesurés (46-0) ; **fiches écrites (2026-11-20), validation en boucle à mener**.
3. ~~Fusion des trois correctifs~~ : faite (`main` `d29aa557`, reprise le 2026-11-20). Reste : la **48-11** dans `main`
   avant la 46-3 ; la maquette de la 46-5 avant sa décision.

## Validation des fiches

- 2026-11-20 — **P1** des fiches 46-1, 46-2, 46-3 (Claude Opus 5.5, contexte frais ; rapports
  `kesh-gate-logs/epic-46/validation/46-{1,2,3}-P1.md`) : 46-1 0/0/4/4, 46-2 0/0/3/3, 46-3 0/0/1/4 (C/H/M/L),
  remédiés. Décisions prises : enveloppe `AvecGroupes<T>` au lieu de champs ajoutés aux structures (C-46-1-4 ; plus
  de collision de la 46-1 avec la 48-11 et la 47-3 dans les littéraux de `pdf.rs`/`csv.rs`) ; un groupe fermé à la main
  reste affiché même nul (46-1 AC-5) ; « hiérarchie masquée » = vue à plat (46-2 C-46-2-2) ; redemande à chaque clic,
  une trace d'audit par demande, assumée (46-2 C-46-2-1) ; ordre 46-3 → 46-2 fixé ; en-tête du PDF sans liste de
  numéros (46-3 C-46-3-1). **À propager à la 46-6a** (en validation par un autre agent au même moment, non touchée
  ici) : T1 par l'enveloppe, et l'AC-3 sur la règle des groupes nuls fermés à la main.
- 2026-11-20 — **P2** des fiches 46-1, 46-2, 46-3 (Claude Sonnet 5.5, contexte frais ; rapports `*-P2.md`) : 46-1 0/0/1/4,
  46-2 0/0/0/2, 46-3 0/0/1/1 (C/H/M/L), remédiés. `AvecGroupes<T, S>` (deux paramètres) ; entrée typée de la
  validation pour le MCP ; garde « rapport vide » des rendus groupés (46-3) ; groupes dissous du plan PME relevés.
  « hiérarchie masquée » = vue à plat : **décidé par le responsable de l'epic le 2026-11-20** (46-2 C-46-2-2).
- 2026-11-20 — **P3 ciblée** (Claude Haiku 5.5, contexte frais, commit `8a7f0f73`) des fiches 46-1, 46-2, 46-3 : 0 C / 0 H / 1 M / 1 L, remédiés. M : la table des fichiers communs gardait `AvecGroupes<T>` (propagation du P2 incomplète) → `AvecGroupes<T, S>` ; L : les noms de champs de D8 renvoient désormais au contrat de la 46-1. **Hors périmètre de cette passe, à propager à la 46-6a et à la 46-6b (fiches d'un autre agent, non touchées)** : `AvecGroupes<T>` à un paramètre aux lignes 24, 40, 69 et 106 de la 46-6a, `AvecGroupes<TrialBalance>` à la ligne 45 de la 46-6b ; à lire `AvecGroupes<T, S>` (C-46-1-4). Rapport : `kesh-gate-logs/epic-46/validation/46-123-P3-ciblee.md`.
- 2026-11-20 — **Décisions du responsable de l'epic** : (1) « hiérarchie masquée » = la liste à plat des comptes, comme l'écran d'aujourd'hui ; (2) le rechargement à chaque clic de la 46-2 (aller-retour API, trace d'audit `report.generated`) est accepté **quel que soit l'état du stockage** (précisé le même jour : la vue est mémorisée dans la mesure où le navigateur le permet) — niveau et groupes ouverts ou fermés retrouvés au retour sur la page : 46-2 AC-10 (`localStorage`, clé par société et par rapport, restaurée avant la première requête).
- 2026-11-21 — **Corrections des fiches 46-3 et 46-6a avant développement**, sur l'API réelle de la 46-1 (`story/46-1`,
  `de0a582f`) et ses revues (P1 L-4, L-5 ; P2 L-6 : paramètres de groupes lus par `groupes_de`, jamais par des champs
  ajoutés à `ExportQuery` ou `ReportQuery`), et sur la 47-3 (point d'insertion de la présentation miroir ; un miroir
  sous un groupe d'un autre type, avertissement non bloquant, décision du responsable de l'Epic 47 du 2026-11-21). Choix
  neuf à confirmer en validation : **à la balance, une ligne nulle ne compte pas pour la dissolution par nature**
  (46-6a C-46-6a-3), d'où une phrase propagée à la 46-6b. Les deux fiches repassent `drafted` ; nouvelle boucle de
  validation (prompt `kesh-gate-logs/s46-specs/validation-p1-prompt.md`).
- 2026-11-21 — **Validation P1 de la nouvelle boucle des 46-3 et 46-6a** (`kesh-gate-logs/s46-specs/validation-p1-rapport.md`) :
  0 C / 0 H / 2 M / 10 L, remédiés ; C-46-6a-3 jugé juste. Décision sur la sortie MCP sans argument neuf : **`groupes:
  null` admis** (`sortie!` n'omet aucun champ), littéral de base inchangé, la clé ajoutée par le test (46-6a
  C-46-6a-5, 46-4 C-46-4-5). La 46-6b repasse en `drafted` (phrase de manuel adossée à C-46-6a-3) ; ligne
  `trial_balance.rs` des fichiers communs corrigée (aucune story). Passe P2 à suivre
  (`kesh-gate-logs/s46-specs/validation-p2-prompt.md`).
