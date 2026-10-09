# Story 15.1a : Le socle du lettrage — index (découpée)

## Status

split

⛔ **CORPS VIDÉ — cette fiche ne contient plus ni décisions, ni critères, ni tâches** *(découpée le
2026-10-09 à la validation P3, C124)*. Elle ne garde que les pointeurs vers ses deux sous-fiches, la
table qui dit où chaque élément est allé, le recompte aux deux bornes, et l'historique des passes
(Change Log). *(Précédents : la 15-1 et la 15-12 ; la définition du statut `split` l'impose.)* Les
fiches sœurs et le registre qui citent « 15-1a R5 », « 15-1a AC5 »… restent justes : **la
numérotation est conservée** dans les sous-fiches (C124).

## Les deux sous-fiches

| ordre | fiche | ce qu'elle porte |
|---|---|---|
| 1 | `15-1a-i-marque-du-lettrage.md` | **La marque** : deux colonnes sur `journal_entry_lines` et le bump `min_required` 0.13.0, le code en lettres, la primitive unique (création, dissolution, modes `Manual`/`System`) et sa règle des périodes, les verrous d'exercice, les routes `/letterings`, l'audit, l'exposition et l'export, les dix clés des refus du lettrage, la documentation de l'API et du manuel propre au lettrage |
| 2 | `15-1a-ii-gardes-du-lettrage.md` | **Les gardes** : le gel des écritures lettrées (`ENTRY_LETTERED`, dernier refus du `PUT`, du `DELETE`, de la dévalidation et du motif d'écran), l'écran de la fiche d'écriture, le lettrage `reversal` de la contre-passation (R6), les réserves du manuel et de l'API sur la modification, les entrées #532 du CHANGELOG |

Ordre : **15-12a → 15-12b → 15-1a-i → 15-1a-ii → 15-1a2 → 15-1b → 15-1c**. ⛔ **Dépendance
résiduelle** : la 15-1a-ii suppose la 15-1a-i mergée ; entre les deux merges, une écriture manuelle
lettrée par l'API reste modifiable et supprimable — **la v0.13.0 ne se tague pas entre les deux**
(C124).

## Pourquoi le découpage

Le déclencheur écrit à **C118** (validation P2 : « si la P3 trouve un défaut né d'un correctif de P2 sur
une règle métier — R7, AC4, AC5, AC8 —, découper selon cette couture avant toute P4 ») est **atteint** :
R3-1 (= F3-1) naît de C113 croisé avec le rang d'AC8, R3-2 de C114, R3-3 du test que C114 a ajouté.
C'est le signal D5 de **recyclage** — le défaut naît du correctif —, non la découverte de défauts
d'origine neufs. Décision de l'orchestrateur, consignée **C124** ; couture de C118 : (i) schéma,
primitive, routes, audit, exports ; (ii) gardes d'AC8, contre-passation R6, frontend de la fiche
d'écriture, manuel de la modification.

## Table de correspondance — où chaque élément est allé (C124)

| élément de la 15-1a | 15-1a-i | 15-1a-ii |
|---|---|---|
| Reprise du 2026-10-08, quatre questions du dégel | ✓ (section commune) | renvoi |
| R1 (schéma, bump), R2 (code), R3 (groupe, primitive), R4 (comptes lettrables), R5 (lignes de pièce) | ✓ | renvois |
| R6 (contre-passation) | renvoi | ✓ |
| R7 (verrous, règle des périodes, interblocages) | ✓ (point 2 réécrit, C125) | renvois |
| AC1–AC7 | ✓ | — |
| AC8 (gel `ENTRY_LETTERED`, écran) | — | ✓ (rang révisé, C126) |
| AC9 (contre-passation) | — | ✓ |
| AC10 (audit) | part i (`created`/`removed` des routes) | part ii (lettrage `reversal`) |
| AC11, AC12, AC14 | ✓ | — |
| AC13 (inventaire, invariant) | part i (colonnes, invariant sur groupes `manual`) | part ii (groupes `reversal`, commentaire vers la garde) |
| AC15 (documentation) | part i (routes, refus du lettrage, champs et colonnes, verrou de période, glossaire, « à lettrer », CHANGELOG *Ajouté* et *Modifié*) | part ii (réserves « lettrée », tableaux du `PUT`/`DELETE`, `:279`, contre-passation, sixième condition du manuel, entrées #532) |
| T0 (relevés au sol) | part i (`EXPLAIN` de l'acte 1, 15-12a mergée) | part ii (`EXPLAIN` de `lettering_guard`, 15-12b, ordre réel des étapes) |
| T1, T2, T3, T6, T7, T8, T9 | ✓ | — |
| T4, T4-bis, T5 | — | ✓ |
| T10 (i18n) | part i (dix clés + lot d'audit) | part ii (`ENTRY_LETTERED`) |
| T11 (documentation), T12 (tests) | part i | part ii |
| Dev Notes : gate, P8, dépendances, règle de découpage, FR86, références | ✓ (P8 ici seulement : la migration y est) | ✓ (sans P8) |

## Recompte aux deux bornes

*(Borne basse : la fiche 15-1a au commit `c9cf51f8`, corps avant `## Change Log`. Borne haute : les deux
sous-fiches au commit de ce découpage, corps avant `## Change Log`. Commandes : `grep -oE '^\*\*AC[0-9]+'`,
`grep -cE '^- \[ \] \*\*T'`, lignes du tableau de T10, et `grep -oE` des noms de tests en
`snake_case`, triés à la main entre tests **neufs** et tests existants cités.)*

| décompte | 15-1a (avant) | 15-1a-i | 15-1a-ii | après | écart, et pourquoi |
|---|---|---|---|---|---|
| critères (numéros AC distincts) | 15 | 13 entrées (AC1–AC7, AC10–AC15) | 5 entrées (AC8, AC9, AC10, AC13, AC15) | **15 numéros**, 18 entrées | 3 critères partagés (AC10, AC13, AC15), écrits « part i / part ii » ; aucun perdu |
| tâches (identifiants distincts) | 14 (T0–T12, T4-bis) | 11 entrées | 7 entrées | **14 identifiants**, 18 entrées | 4 tâches partagées (T0, T10, T11, T12) ; aucune perdue |
| clés i18n (tableau de T10) | 11 | 10 | 1 | **11** | — ; le lot d'audit (1 entité, 2 actions) reste à la 15-1a-i |
| tests **neufs** nommés | 15 | 14 repris + 2 neufs = 16 | 1 repris + 5 neufs = 6 | **22** | +7 de la remédiation P3 : `archived_bank_account_keeps_its_account_unletterable` (F3-7), `lettering_locks_fiscal_years_in_date_order_not_id_order` (R3-3), et dans la 15-1a-ii `delete_of_…`, `update_of_…_in_a_locked_period_…`, `blocker_of_…` (R3-1), `update_of_a_lettered_entry_with_a_stale_version_says_conflict`, `entry_lettered_refusal_leads_to_a_dissolution_that_succeeds` (C126) |
| mutations nommées | 2 (paire C117 « permuter » ; `:197` du garde-fou d'inventaire) | 2 (`:197` ; « trier par `id` ») | 4 (« la marque avant le verrou de période » × 3 chemins ; paire C117) | **6** | +4 : une par chemin de précédence (R3-1) et celle du test d'ordre (R3-3) |
| puces des Dev Notes | 6 | 6 | 5 | — | P8 seulement dans la 15-1a-i (la migration y est) ; les autres puces sont réécrites pour chaque moitié |

⚠️ Les quinze tests neufs de la borne basse sont : six d'AC4, trois du point 3 d'AC5, trois d'AC10,
`lettering_invariants` (AC13), deux d'AC14. Les tests **existants** que la story met à jour ou cite
(`downgrade_protection_*`, `every_data_backfill_migration_is_triaged`,
`the_inventory_guard_turns_red_on_each_mutation`, les partitions du registre…) ne sont pas comptés.

## Change Log

### Validation P6 — 2026-10-09 (15-1a-ii : Sonnet ×2 ; remédiation Opus 5.5)

**15-1a-ii** : deux rapports (`target/gate-logs/15-1a-ii-p6-{R,F}.md`, prompt `3acf1860`) — 0 HIGH,
**1 MEDIUM**, 7 LOW distincts, **sans changement de règle** : deux sites d'`audit_route_registry.rs`
(`:82`, `:201`) qui nient encore le cycle lettrage ↔ `DELETE`, résidu de la propagation de F5-2 faite
par la formulation — relevé désormais écrit **par la valeur** et rejoué (8 lignes, 6 inscrites, 2
étrangères triées) ; LOW : doc frontend de `reverseJournalEntry`, réserve « chiffres relus après la
15-12b » sur C132, tests en module de `reverse_in_tx`, origine sous période verrouillée au test (c),
ponctuation du manuel, ordre de l'union `ModificationBlocker`. **Trend du socle** : P1 2 HIGH / 15
MEDIUM → P2 0 / 5 → P3 0 / 4 → P4 0 / 5 → P5 0 / 3 → P6 0 / 1. Aucun choix neuf au registre ; détail
au Change Log de la 15-1a-ii.

### Validation P5 — 2026-10-09 (15-1a-i : ciblée Haiku ; 15-1a-ii : Opus 5.5 ×2 ; remédiation Opus 5.5)

**15-1a-i** : passe ciblée close à 0 (voir sa fiche, `a8dd72d1`). **15-1a-ii** : deux rapports
(`target/gate-logs/15-1a-ii-p5-{R,F}.md`, prompts `c6a88f03`) — 0 HIGH, **3 MEDIUM**, 10 LOW distincts,
tous d'**inventaire ou de propagation**, **sans changement de règle** : textes d'écran du bilan
d'ouverture qui promettent la modification sans réserve (R5-1, et le contrôle final étendu aux
catalogues i18n et aux replis), phrase du manuel utilisateur « l'écriture d'origine n'est pas touchée »
(F5-1, doctrine C129), textes qui nient le cycle lettrage ↔ `PUT`/`DELETE` ou décrivent un ordre de
verrous incomplet (F5-2, Pattern 5 et trois doc-comments). **Trend du socle** : P1 2 HIGH / 15 MEDIUM →
P2 0 / 5 → P3 0 / 4 → P4 0 / 5 → P5 0 / 3. Décision **C132** (décompte des refus de la dévalidation) ;
détail au Change Log de la 15-1a-ii.

**Recompte après P5** (corps des sous-fiches, avant `## Change Log`) : critères et tâches **inchangés** ;
clés i18n neuves **11** (10 + 1), plus **5 clés existantes réécrites** (2 dans la 15-1a-i, C130 ; 3 dans
la 15-1a-ii, C129 et R5-1) ; tests neufs nommés **29** (16 + 13, inchangé) ; mutations nommées **11**
(3 + 8 — la 15-1a-ii en comptait 7 en P4 par erreur, recompté).

### Validation P4 des deux sous-fiches — 2026-10-09 (Sonnet 5.5 ×2 par fiche ; remédiation Opus 5.5)

Quatre rapports (`target/gate-logs/15-1a-i-p4-{R,F}.md`, `15-1a-ii-p4-{R,F}.md`, prompts versionnés
`b53f7278`). **15-1a-i** : 0 HIGH, **2 MEDIUM**, 11 LOW distincts. **15-1a-ii** : 0 HIGH, **4 MEDIUM**, 12
LOW distincts. Une MEDIUM est commune aux deux (nom des exercices en mode `System` — C128) : **5 MEDIUM
distinctes** pour le socle. **Trend du socle** : P1 2 HIGH / 15 MEDIUM → P2 0 / 5 → P3 0 / 4 → P4 0 / 5.
⚠️ **Signal D5, déclaré au Project Lead** : la sévérité ne décroît pas (4 → 5 MEDIUM), mais une seule
MEDIUM naît d'un correctif (C127 × C125) ; les quatre autres sont des défauts d'origine (inventaire de la
promesse « à lettrer », test de correspondance muet, manuel d'administration, origine « intacte » que R6
marque). Pas de nouveau découpage proposé. Décisions **C128 à C131** ; détail au Change Log de chaque
sous-fiche ; report à la 15-1a2 (« Reçu », points 18 à 20).

**Recompte après P4** (corps des sous-fiches, avant `## Change Log`) : critères et tâches **inchangés**
(15 numéros / 18 entrées ; 14 identifiants / 18 entrées) ; clés i18n neuves **11** (10 + 1), plus **3 clés
existantes réécrites** (2 dans la 15-1a-i, C130 ; 1 dans la 15-1a-ii, C129) ; tests neufs nommés **29**
(16 + 13 — les sept noms ajoutés en P4 désignent des tests d'AC9 et la paire C117, déjà décrits) ;
mutations nommées **10** (3 + 7).

### Validation P3 et découpage — 2026-10-09 (Opus 5.5 ×2 ; remédiation Opus 5.5)

**Deux lentilles, Opus toutes deux** (prompt versionné `15-1a-validate-prompt-p3.md`) : R (chasseur de
régressions) **0 HIGH, 3 MEDIUM, 10 LOW** ; F (adversaire plein périmètre) **0 HIGH, 2 MEDIUM, 6 LOW**
(`target/gate-logs/15-1a-p3-{R,F}.md`). Recoupement : R3-1 = F3-1. **Distincts : 0 HIGH, 4 MEDIUM**
(R3-1/F3-1, R3-2, R3-3, F3-2) **et 16 LOW** (R L1–L10, F3-3 à F3-8). **Trend** : P1 **2 HIGH / 15
MEDIUM** → P2 **0 HIGH / 5 MEDIUM / 16 LOW** → P3 **0 HIGH / 4 MEDIUM / 16 LOW** (distincts). Modèles :
P1 Opus ×2, P2 Sonnet ×2, P3 Opus ×2 (rotation D6).

⛔ **Signal D5 — recyclage** : trois des quatre MEDIUM naissent de correctifs de P2 sur des règles métier
(C113 croisé avec AC8 ; C114 ; le test de C114). Le déclencheur de C118 est atteint ; **découpage**
décidé par l'orchestrateur (C124), avant toute P4. Remédiation appliquée **dans les sous-fiches** :
C125 (verrous d'exercice un par un, bornés au groupe, postérieur clos sans verrou — R3-2, aligné sur
C119), C126 (`ENTRY_LETTERED` en dernier — R3-1), C127 (compte bancaire archivé, exercice des lignes,
rubriques du CHANGELOG — F3-7, F3-8, F3-2) ; R3-3 par un test d'ordre discriminant (15-1a-i, T12) ;
tous les LOW (détail au Change Log de chaque sous-fiche). Propagation : `epics.md` (règle de C113 et
découpage, L4), `sprint-status.yaml`, index `15-1-lettrage.md`, fiche d'epic, renvois des 15-1a2, 15-1b,
15-1c ; les fiches 15-12* ne sont **pas** modifiées (L5 et l'étape « 3-ter-bis » de la 15-12b, portés à
l'orchestrateur).

### Validation P2 — 2026-10-09 (Sonnet 5.5 ×2, contexte frais) — remédiation (Opus 5.5)

**Deux lentilles, Sonnet toutes deux** (prompt versionné `15-1a-validate-prompt-p2.md`) : R
(chasseur de régressions) **0 HIGH, 2 MEDIUM, 7 LOW** ; F (adversaire plein périmètre) **0 HIGH,
4 MEDIUM, 11 LOW** (`target/gate-logs/15-1a-p2-R.md`, `…-F.md`). Recoupements : R2-1 = F-2 ;
R2-2 = F-5 (MEDIUM chez R, LOW chez F ; F-5 porte aussi la clôture « non rejouée » et le cycle avec la
contre-passation, qui recoupe R2-7) ; R2-6 ≈ F-7. **Distincts après fusion : 0 HIGH, 5 MEDIUM**
(R2-1/F-2, R2-2/F-5, F-1, F-3, F-4) **et 16 LOW** (les sept de R, R2-3 à R2-9, plus F-6, F-8 à
F-15). **Trend** : P1 **2 HIGH / 15 MEDIUM** distincts → P2 **0 HIGH / 5 MEDIUM / 16 LOW**.
Décisions consignées **C113 à C118** (les deux premières fixées par l'orchestrateur : verrou de
période = exercice clos pour le lettrage ; ordre des verrous d'exercice garanti par le parcours).

| finding | décision | où |
|---|---|---|
| F-1 | Motif du bump réécrit : tout binaire publié depuis la v0.10.0, et pas « la v0.12.1 seule » ; en-tête de migration **arrêté** dans la fiche (P8) ; constat écrit que `sqlx` (`VersionMissing`) et l'import (`unknownColumns`) refusent déjà — le bump rend le refus explicite | R1, AC1, T1, C115 |
| R2-1 = F-2 | Les deux tests qui rougissent au bump nommés (`migrations_upgrade_path.rs:509`, `:524`) ; `rejects_old_binary` ramené à une non-régression | AC1, T1, T12 |
| R2-2 = F-5 | Exercices verrouillés par **parcours ascendant** de `(company_id, start_date)` à partir du plus ancien du groupe ; `EXPLAIN` en T0, repli « un par un » écrit ; test aux `id` inversés ; clôture « non rejouée » corrigée (15-12a AC 6) ; contre-passation nommée dans les cycles | R7, AC5, T0, T3, T12, C114 |
| F-3 + état hérité (reçu de la 15-12) | Règle des **périodes** : une ligne est « en période ouverte » si exercice ouvert, aucun postérieur clos, date après le verrou ; refus `LETTERING_ALL_LINES_IN_CLOSED_PERIODS` (remplace `LETTERING_FISCAL_YEARS_CLOSED`) ; manuel `:578-583` ; six + trois tests nommés | R7, AC3, AC4, AC5, AC15, T10, C113 |
| F-4 | `reverse_in_tx_inner` relit les lignes après lettrage ; AC9 (a) lit le corps du `201` | R6, AC9, T5, C116 |
| Reçu de la 15-12 (`9b403aaf`) | Prérequis **15-12a** ; ordre 15-12a → 15-12b → 15-1a ; précédence 2-bis → 3-ter-bis dans `delete_in_tx` écrite et testée par la seconde à merger | Status, Story, Dev Notes, AC8, C117 |

**LOW appliqués** : R2-3 (une fonction pure par cause, appelée à son rang), R2-4 (`errors.rs:206`,
chemins préfixés, six fichiers `0.12.1`), R2-5 (six `query_as`, `:1824/:1828`), R2-6 = F-7 (sites
d'`api-external.md` en tableau, `routes/journal_entries.rs:166`, `JournalEntryForm.edit.test.ts:132`,
grep par le motif voisin), R2-7 (cycle avec la contre-passation, décompte des routes rejouées), R2-8
(lot des libellés d'audit dans les recomptes i18n), R2-9 (texte neutre de
`LETTERING_LINE_OWNED_BY_DOCUMENT`), F-6 (requête de `lettering_guard` sans `ORDER BY … LIMIT`,
`EXPLAIN`), F-8 (arithmétique vérifiée, borne `i64::MAX`, tests), F-9 (exceptions de R3 nommées, T9
étendu à `INSERT`, périmètre `src/` hors `#[cfg(test)]`), F-10 (fixture sans rôle), F-11 (limite de
durée nommée, R7 et AC11), F-12 (coût de R6 écrit), F-13 (règle de découpage relue : pas de découpage,
couture et déclencheur écrits — C118), F-14 (section *Ajouté* à créer en tête), F-15 (glossaire
réécrit, parenthèse du message assumée). Axe que F déclarait non exercé, repris : montage des routes
au-dessus du `route_layer` (T6).

**Reporté aux fiches sœurs** (sections « Reçu de la 15-1a », sans réécriture) : 15-1a2 (points 9 à
13), 15-1b et 15-1c (sections neuves) ; index `15-1-lettrage.md` (état hérité gardé ici, C113-C118).
**Décomptes recomptés depuis ce fichier** (`grep` sur le corps, avant `## Change Log`) : **15
critères** (AC1–AC15), **14 tâches** (T0–T12 et T4-bis ; T0 est neuve), **11 clés i18n** au tableau
de T10 (plus le lot d'audit, hors tableau).

⚠️ **À l'orchestrateur** : (1) **fiche 15-12b** (non modifiée ici, en validation) : si la 15-1a est
mergée avant elle, la paire de précédence de `delete_in_tx` lui revient (C117) ; (2) le constat de
C115 sur `sqlx` (`ignore_missing = false`) touche la prémisse de la § « Migration breaking policy »
du `CLAUDE.md` — à porter à Guy, hors story ; (3) la révision de D3, désormais étendue au verrou de
période, reste à présenter à Guy ; (4) **passe P3 : protocole complet**, Opus (rotation D6) — la
remédiation change une règle métier (R7, AC4, AC5) et la séquence des verrous ; déclencheur de
découpage écrit à C118.

### Validation P1 après reprise — 2026-10-08 (Opus 5.5 ×2, contexte frais) — remédiation

**Deux lentilles, Opus toutes deux** : R (auditeur d'acceptation) **2 HIGH, 10 MEDIUM, 9 LOW** ;
F (adversaire plein périmètre) **1 HIGH, 9 MEDIUM, 7 LOW** (`target/gate-logs/15-1a-p1-R.md`,
`…-F.md`). Recoupements : R-1 = F1, R-3 = F2, R-2 = F3, R-5 = F4, R-7 = F5, R-6 = F7 ; R-11 ≈ F-L1,
R-12 = F-L1, R-13 = corollaire de F4, R-4 = F-L2, R-14 ≈ F-L3, R-16 = F-L6. ⚠️ Le bilan de F
annonce 9 MEDIUM mais en **liste dix** (F2 à F11) ; le décompte ci-dessous part des listes.
**Distincts après fusion** : **2 HIGH** (R-1/F1 ; R-2, HIGH chez R, = F3, MEDIUM chez F) et
**15 MEDIUM** — les dix de R (R-3 à R-12) plus les cinq de F sans jumeau chez R (F6, F8, F9, F10,
F11 ; F2, F4, F5, F7 sont R-3, R-5, R-7, R-6). Trend : relecture
de reprise → **P1 : 2 HIGH / 15 MEDIUM distincts**. Décisions de l'orchestrateur, en autonomie,
consignées **C101 à C106** (révisent C90, C94, C96, C97) :

| finding | décision | où |
|---|---|---|
| R-1 = F1 (HIGH) | Bump `min_required = '0.13.0'` dans **cette** migration, motif réécrit au plus juste (la v0.12.1 ne modifie aucune écriture manuelle ; elle annulerait un règlement sans dissoudre le groupe de la pièce) ; P2-bis (dix crates à 0.13.0, même commit) ; P7 (`EXEMPT_MIGRATIONS`, `Durable`) ; gate runtime ; effet sur `prepare-release.sh` écrit (refus « version identique » avant le pré-vol) | R1, AC1, T1, C101 |
| R-2 = F3 (HIGH) | `ModificationGuard::Lettered`, lu sans verrou dans `modification_blocker` ; code d'écran `ENTRY_LETTERED`, frontend (union, libellé, issue du refus, tests), « onze » → « douze », `api-external.md:229/:261` ; module frontend compté | AC8, T4, T4-bis, C102 |
| R-3 = F2 | La garde de lettrage est une étape **inconditionnelle** de `delete_in_tx` (hors `enforce_ownership`) : la dévalidation est couverte ; test `delete_in_tx(…, false)` | AC8, T4, T12, C102 |
| R-5 = F4 (+ R-13) | Premier acte **verrouillant** joint (lignes et en-têtes), aucune lecture ordinaire avant ; compte d'`UPDATE` inattendu → 409 `LETTERING_CONCURRENT_CHANGE`, jamais `Invariant` ; motif de la lecture verrouillante d'AC8 réécrit | R7, AC3, AC6, AC8, C103 |
| R-6 = F7 (+ R-4) | Exercices `FOR UPDATE ORDER BY start_date, id` en mode `Manual` ; **aucun** verrou d'exercice en mode `System` (l'appelant tient l'exercice ouvert et le passe) — affinement de la décision 7 : reprendre les exercices de l'origine après celui du jour inverserait l'ordre contre la clôture de la 15-12, non rejouée | R7, AC5, C103 |
| R-7 = F5 | La 15-12 passe **avant** la 15-1a ; ordre écrit ici, dans l'index `15-1-lettrage.md`, dans `epics.md` et au `sprint-status.yaml` | Dev Notes, C105 |
| F6 | Règle **symétrique** : lettrer comme délettrer exige ≥ 1 ligne sur exercice ouvert ; révise D3 (arbitrage de Guy d'août — signalé pour sa revue) | R7, AC4, C105 |
| F8 | Lettrabilité exigée à la création seule ; groupes intacts et dissolubles après retypage ou rattachement bancaire ; test | R3, AC5, AC9 (e), AC13, C104 |
| F9 | Groupe `reversal` contenant une ligne de pièce : non dissoluble à la main (`LETTERING_LINE_OWNED_BY_DOCUMENT`) | R6, AC5, AC9 (d), C106 |
| F10 | Manuel `:1169/:1696` et `api-external.md:325/:386` ne promettent plus « à lettrer » ; cas renvoyé à la 15-1a2 (section « Reçu de la 15-1a ») | R5, AC15, C106 |
| F11 (+ R-18) | AC15 nomme tous les sites (`.tex` : 384, 481/483, 562, 628, 708, 744, 758, 2209 ; API : 223, 229, 255, 261), PDF régénéré et contrôlé aplati | AC15, T11 |
| R-8 | `epics.md:1363/1366/1368` alignés sur la reprise | `epics.md` |
| R-9 | Tests ajoutés pour AC6, AC10, AC14 ; geste d'AC13 rattaché à T8 | T8, T12 |
| R-10 | Onze clés nommées, ventilées, avec texte FR | T10 |
| R-11, R-12 (= F-L1) | `kesh-report` retiré (agrégats) ; inventaire réel des sites `JournalEntryLine` ; décompte de découpage : **5** crates/packages, signal déclaré | AC14, Dev Notes |

**LOW appliqués** : R-14/F-L3 (acteur `for_actor`, clé d'API ; écart de la contre-passation nommé),
R-15 (appariement par position), R-16/F-L6 (citations `reconciliation_cancel.rs:370`, `:2357`
retirée), R-17 (FR86 interprété, Dev Notes), R-19 (restauration d'une sauvegarde antérieure, R1),
R-20 (export global, AC14), R-21 (sort des groupes — C104 ; noms des handlers, AC6/AC12), F-L4
(doc-comment `:181`), F-L5 (verrou de période : permis, R7), F-L7 (lecture des lignes de l'origine
pour R6). **Décomptes recomptés depuis ce fichier** (`grep -c` sur le corps, avant `## Change Log`) :
**15 critères** (AC1–AC15), **13 tâches** (T1–T12 et T4-bis), **11 clés i18n** au tableau de T10.

⚠️ **À l'orchestrateur** : (1) `scripts/prepare-release.sh` refusera `0.13.0` (versions déjà
bumpées) **avant** son pré-vol — la release v0.13.0 devra dater le CHANGELOG et lancer le contrôle
des exemptions périssables à la main, ou le script apprendre ce cas (C101) ; (2) la révision de D3
est à présenter à Guy ; (3) **passe P2** : protocole complet (la remédiation touche R1, R7, AC3–AC6,
AC8 — plusieurs règles métier), Sonnet ×2 (rotation D6).

### Reprise du 2026-10-08 — réécriture contre le modèle réel (Opus 5.5, en autonomie)

Corps **entièrement réécrit** ; les entrées ci-dessous en gardent l'historique. Ce que la reprise
change, et pourquoi (registre C90–C99) :

- **Porteur** : la table `letterings` (arbitrage du 2026-08-25) est **abandonnée** au profit de
  deux colonnes sur `journal_entry_lines` — une table neuve rendrait inimportables toutes les
  sauvegardes antérieures, contrainte apparue avec l'import d'installation (#386) et déjà
  appliquée par la 25-4-d2a. **La clé du groupe est le plus petit id de ligne** : le compteur, son
  verrou et ses trois passes de défauts (P5-1, P6-1, P8-3) disparaissent avec lui.
- **Forme** : la **paire** devient un **groupe** à somme nulle — imposé par `invoice_settlements`
  (règlements multiples, solde du reste).
- **D3** révisée étroitement (C94) ; **AC7 (ii)** d'août abandonnée (C97) ; la décision ouverte
  **P8-2** est tranchée (conduite (b) **et** lecture verrouillante de la garde) ; **T5-bis** sans
  objet (plus de table à exporter).
- **Nouveau** : la contre-passation lettre ce qui est libre (R6) ; les lignes de pièce sont
  exclues du lettrage manuel (R5) ; comptes lettrables définis par type (R4) ; la sous-story
  **15-1a2** (lettrage des pièces) est créée.
- Décomptes : **15 critères** (AC1–AC15), **12 tâches** (T1–T12), recomptés depuis ce fichier.

*Entrées antérieures à la reprise — le corps qu'elles décrivent a été remplacé :*


### Passe 8 de `validate` — 2026-08-25 (Opus, contexte frais) — **DERNIÈRE PASSE**

**2 HIGH, 4 MEDIUM, 4 LOW.** Le plafond de 8 passes de la § *Review Iteration Rule* est
**atteint** : il n'y aura pas de passe 9.

⚠️ **Ce que huit passes ont établi, et c'est le motif de cette story** : les deux HIGH sont,
pour la **troisième fois consécutive**, des défauts de **verrouillage** — et l'un des deux naît
du correctif de la passe précédente. Sur l'ensemble, **quatre des cinq findings les plus
graves sont nés d'un patch de remédiation**, aucun de la conception d'origine.

⛔ **P8-1 (HIGH) — « le sentinel les ferme toutes les deux » était FAUX.** Il ferme l'ABBA
entre deux délettrages ; il ne ferme **rien** de la clôture concurrente, car
**`fiscal_years::close` ne prend aucun verrou sur `companies`** — sa première instruction est
son `UPDATE`, et `docs/MULTI-TENANT-SCOPING-PATTERNS.md` le confirme (*« fiscal_years only,
single table »*). Deux transactions ne se sérialisent que sur une ressource **commune** ; la
seule ici est la ligne `fiscal_years`. → AC6 exige désormais le `SELECT … FOR UPDATE` sur les
deux lignes d'exercice : **c'est ce verrou-là, et lui seul**, qui sérialise avec `close`. Le
sentinel ne fait qu'**ordonner**. Sans lui, une créance se rouvre sur un exercice clos **sans
erreur ni trace**.

⛔ **P8-2 (HIGH) — laissé en DÉCISION OUVERTE**, budget épuisé : le lettrage sera le premier
écrivain de `journal_entry_lines` à ne pas tenir le verrou d'en-tête, et `update` fige son
instantané sur une **lecture nue antérieure** à ce verrou. Un lettrage qui commit entre les
deux est **invisible** à la garde d'AC7. Deux conduites praticables, l'arbitrage engageant
l'ordre de verrou global du dépôt — voir le § qui précède AC8.

**Quatre MEDIUM, dont deux qui corrigent des affirmations FAUSSES de passes précédentes :**

| | défaut | remède |
|---|---|---|
| **P8-3** | le `FOR UPDATE` du `MAX(seq)` avait **disparu** entre deux sites de la spec, et T4 poussait à placer le chargement des lignes **avant** le sentinel — l'instantané `REPEATABLE READ` se fige à la première lecture nue | `FOR UPDATE` rétabli ; le sentinel est le **premier énoncé** |
| **P8-4** | ⚠️ **« la garde-modèle des `invoices` est structurelle » était FAUX** : elle compare le header à une **chaîne littérale** (`csv_tables.rs:1036`) et **ne rougit pas** quand un champ entre dans la struct | la décision reste, son motif est réécrit ; T5 dit ce que la garde protège **et ce qu'elle ne protège pas** |
| **P8-5** | **le délettrage n'avait aucun contrat d'entrée** — or son chemin de scoping, donc le test d'AC11, en dépend entièrement | T4 nomme le contrat des deux routes |
| **P8-6** | trois cibles de verrou neuves entraient dans le dépôt **sans être placées dans l'ordre global**, que `MULTI-TENANT-SCOPING-PATTERNS.md` **exige** de tenir à jour — « respecter l'ordre global » n'était pas exécutable | T4 impose l'inscription au tableau du Pattern 5 |

**Quatre LOW** : `letterings` absente de l'**export de souveraineté** — laissé en **décision
ouverte** (T5-bis), l'utilisateur obtiendrait sinon des entiers opaques sans le code qu'il voit
à l'écran ; une citation désignant le mauvais `projects.rs` (il y en a deux) ; un fragment de
phrase mort dans T7 ; et **le décompte de critères était faux — 13, pas 14**, dans un fichier
qui invoque la règle de recompte.

**Vérifié et réfuté** : faire entrer `lettering_id` dans la struct partagée **n'a aucun effet
de bord** — `is_no_op_change` compare champ par champ, `entry_snapshot_json` construit son JSON
à la main, l'API passe par un DTO distinct, et `sample_line()` casse **bruyamment** à la
compilation ; le 404 d'AC11 n'entre pas en conflit avec AC7/AC8, qui sont des gardes internes ;
`audit_log.entity_type` est un `VARCHAR` libre, AC13 n'a **aucune dépendance i18n cachée** ; et
`delete_all_by_company` — troisième chemin de destruction, échappé au recensement de la
passe 1 — n'a **que des appelants de test**.

**Verdict de la passe : la spec ne part pas en développement en l'état**, et **une passe 9 n'y
changerait rien**. Ce qui reste n'est plus de la détection — les défauts sont ici, prouvés,
avec leurs lignes — mais **trois décisions humaines** : la discipline de lecture verrouillée
(à porter aussi dans `MULTI-TENANT-SCOPING-PATTERNS.md`), le côté où se pose le verrou
(P8-2), et l'export de souveraineté (P8-7).

### Passe 7 de `validate` — 2026-08-25 (Sonnet, contexte frais)

**0 CRITICAL, 1 HIGH, 2 MEDIUM, 1 LOW.**

⛔ **P7-1 (HIGH) — la même famille de défaut que la passe 6 venait de fermer, mais sur AC6 :
la décision comptable centrale de la story.** La passe 6 avait spécifié la concurrence de la
**création** du code (AC3) et laissé le **délettrage** sans mécanisme.

⚠️ **Cette story est la PREMIÈRE du dépôt à devoir vérifier DEUX exercices dans une seule
transaction.** Tout le code existant n'en verrouille qu'un — une écriture n'appartient qu'à un
exercice — et le lettrage à cheval qu'AC5 organise brise cette hypothèse. Les deux issues
qu'un développeur aurait prises sont **toutes deux mauvaises** : verrouiller les deux
`fiscal_years` dans l'ordre d'arrivée → **ABBA** entre deux délettrages croisés, avec un 1213
qui remonte tel quel puisque `retry_on_deadlock` n'a aucun site d'appel ; ne pas verrouiller
→ le contrôle « les deux exercices sont ouverts » n'est plus sérialisé avec
`fiscal_years::close`, et **D3 est violée**.

→ AC6 prend le **même sentinel** que le lettrage. Il sérialise tout le trafic d'une société,
ce qui rend l'ordre des verrous suivants sans importance — idiome écrit du dépôt,
`docs/MULTI-TENANT-SCOPING-PATTERNS.md` **Pattern 5**, que la passe 6 avait cité **pour AC3
sans en tirer la conséquence pour AC6**.

**P7-2 (MEDIUM) — l'oracle qu'AC11 venait de fermer pouvait rouvrir par un autre canal.** La
spec disait *que* le refus cross-tenant est un 404 indiscernable, jamais *quand* ce contrôle
s'exécute. Charger les données de la ligne avant de vérifier l'appartenance rendrait un **409
distinct** — « comptes différents », « déjà lettrée » — révélant l'existence **et un
attribut**. Pire que l'oracle fermé une passe plus tôt. → T4 impose la requête unique scopée
et le 404 **avant** toute autre garde, comme le précédent `products.rs:343` l'écrit.

**P7-4 (MEDIUM) — la garde CSV pouvait rester verte sans jamais exporter la marque.** La
garde-modèle des `invoices` est **structurelle** : elle compare le header aux champs de la
**struct**, pas au schéma. Exposer `lettering_id` par une lecture dédiée, sans toucher le type
partagé, l'aurait laissée verte tout en n'exportant rien. → **Tranché** : `lettering_id` entre
dans `JournalEntryLine`, `LINE_COLUMNS` et l'export dès cette story — l'export existe pour
que l'utilisateur **vérifie ce que Kesh affirme**, l'objet même de la story.

**P7-3 (LOW) — le jumeau resté dans `epics.md`.** La passe 6 avait corrigé la ligne 1368
(colonne `lettering_code` écartée) et laissé la 1363 — « deux **ou plusieurs** écritures » —
que D2 contredit, **dans le même paragraphe**. C'est mot pour mot le mode d'échec de la
§ *Propagation post-patch* : corriger un site et laisser son jumeau. Corrigé.

**Vérifié et réfuté** : la croissance non bornée de `letterings` est cohérente avec
`audit_log` (append-only assumé) ; aucune entrée orpheline possible après rollback, tout étant
dans la même transaction ; l'ordre dans `reset_demo` est indifférent (`FOREIGN_KEY_CHECKS=0`),
contrairement à `TABLES_TO_TRUNCATE` ; les mentions résiduelles dans le fichier de kickoff
sont un **instantané antérieur** aux décisions, pas un artefact vivant ; et **le décompte de
sept messages est exact**, recompté depuis la ventilation — 2+1+1+1+1+1.

**Verdict : passe 8 due — la dernière avant le plafond de budget.**

### Passe 6 de `validate` — 2026-08-25 (Opus, contexte frais)

**0 CRITICAL, 1 HIGH, 5 MEDIUM, 3 LOW.** La sévérité décroît (`CRIT → HIGH`) : le critère de
re-split n'est pas déclenché.

⛔ **P6-1 (HIGH) — le correctif de la passe 5 reproduisait, sur un autre plan, la faute qu'il
corrigeait.** Il avait repris de la formule d'`entry_number` **ce qui se voit** — le
`MAX(…) FOR UPDATE` — et laissé **ce qui la fait fonctionner** : le verrou de ligne pris en
amont (`SELECT fiscal_years … FOR UPDATE`, `journal_entries.rs:191`).

`letterings` n'a **aucune ligne préexistante à verrouiller** pour une société encore vierge
de lettrage — et c'est la fixture naturelle du test de concurrence qu'AC3 exige. Deux
transactions y prennent des gap locks **compatibles**, calculent le même `seq`, puis
deadlockent à l'`INSERT`. ⚠️ **Le dépôt porte déjà ce diagnostic écrit**, à propos du même
geste : `email_templates.rs:16-23` — *« un gap lock n'empêche PAS une autre transaction de
tenir aussi son propre gap lock compatible sur la même lacune […] risquent donc de
deadlocker »*.

Et le filet annoncé ne rattrape rien : **un deadlock est un 1213, pas un 1062**. Aucune
contrainte d'unicité ne le voit. Le dépôt a un `retry_on_deadlock` — vérifié : **aucun site
d'appel**, la parade existe et n'est branchée nulle part.

→ AC3 nomme désormais le mécanisme **en deux temps** : sentinel `companies FOR UPDATE`
(Pattern 5, idiome du dépôt), **puis** `MAX(seq)`. Plus l'ordre de verrou global
(`companies → projects → fiscal_years`) et **ce que le test doit constater** — sans quoi son
auteur, voyant un deadlock intermittent, aurait affaibli l'assertion.

**Cinq MEDIUM, dont trois oublis de propagation que la story elle-même prétendait éviter :**

| | défaut | remède |
|---|---|---|
| **P6-2** | *« Le `.keshbackup` n'est pas concerné »* — **vrai pour les colonnes, faux pour les tables** : `TABLES_TO_TRUNCATE` est **codé en dur** et ordonné enfants → parents | T5 : inscrire `letterings` à sa place FK-correcte ; le test compare des listes **triées** et ne verrait pas une position fausse ; et **tout backup antérieur devient non importable** — à dire |
| **P6-3** | **le squash de schéma de test n'était pas mentionné** — T1 anticipait P5, P6, P7 et le compteur d'`upgrade_path`, et oubliait le geste le plus mécanique. Sans `regen-test-schema.sh`, `letterings` n'existe dans **aucune** des ~1100 bases `#[sqlx::test]` | ligne ajoutée à T1 |
| **P6-4** | **le message de refus d'AC11 contredisait la garde anti-IDOR** qu'AC11 pose : nommer « lignes d'une autre société » rend un **oracle d'existence**. La convention du dépôt est écrite dans le code — *« indiscernable d'un compte inexistant »*, 404 | AC11 : verdict indiscernable, **aucune clé** ; T7 repasse de huit à **sept** messages |
| **P6-5** | **délettrer supprimait l'entrée, donc `MAX(seq)` reculait et le code était RÉÉMIS** — un `F` imprimé ou dicté reviendrait sur une paire sans rapport, et l'audit porterait deux lettrages sous une même désignation | `letterings` est **append-only** : délettrer met les `lettering_id` à `NULL` et laisse l'entrée |
| **P6-6** | **D2 n'était nommée par aucune des trois fiches du split** — septième récidive du geste. Et `epics.md:1368` prescrivait encore la colonne `lettering_code` écartée | D2 héritée et citée par AC12 ; `epics.md` corrigé |

**Trois LOW** : l'index se déclare par un `CREATE INDEX` séparé, conforme au seul précédent
d'`ALTER TABLE journal_entry_lines` du dépôt ; deux `CHECK` ajoutés (`seq > 0`, code non
vide) comme les tables voisines en portent ; le commentaire de `code` ne dit plus « saisie »,
`utf8mb4_bin` étant octet-exact. Et **P6-7 : la collision est à la vingt-huitième lettre, pas
la vingt-septième** — après `{A…Z}` le successeur `AA` est juste ; c'est après `{A…Z, AA}` que
`MAX` rend encore `Z`. Le diagnostic de la passe 5 restait entier, seul son ordinal était
décalé.

**Vérifié et réfuté — dont ce qui rassure sur l'arbitrage** : les six cas limites de la
conversion sont **exacts, recalculés** (`26→Z`, `27→AA`, `52→AZ`, `53→BA`, `702→ZZ`,
`703→AAA`) ; **`VARCHAR(16)` couvre tout le domaine `BIGINT`** — `seq = 2⁶³−1` donne
`CRPXNLSKVLJFHG`, quatorze caractères ; le `RESTRICT` sur `lettering_id` ne bloque rien
qu'AC8 attende en succès ; `created_by NOT NULL … RESTRICT` ne casse aucun chemin existant ;
et les compteurs d'idempotence sont sains — **61 partout**, T1 n'hérite rien à réparer.

**Verdict : passe 7 due.**

### Passe 5 — contrôle d'arbitrage — 2026-08-25 (Haiku, contexte frais)

⛔ **1 CRITICAL, 1 HIGH, 2 MEDIUM, 1 LOW.** La passe était due parce que l'arbitrage fixait
une règle structurante **après** la convergence de la passe 4. Elle a bien fait de l'être.

**P5-1 (CRITICAL) — le compteur aurait été FAUX à la vingt-septième lettre.** Le § *Décisions
tranchées* affirmait que le gap lock d'`entry_number` « se transpose à l'identique ». Il ne se
transpose pas : `entry_number` est un **entier**, `code` est du **texte**, et `MAX()` sur du
texte trie **lexicographiquement**. Vérifié : `MAX('A','B','Z','AA','AB','AZ','BA')` rend
**`Z`**, quand la séquence bijective attend `BA`.

⚠️ **Le défaut n'aurait pas été muet — il aurait été bruyant et tardif** : après `Z`, le
compteur serait reparti sur des valeurs déjà prises, et `uq_letterings_company_code` aurait
fait échouer **tout lettrage** de la société. En développement, avec une société de
démonstration à moins de vingt-six lettrages, **rien ne l'aurait révélé**.

**Le correctif ajoute une colonne `seq BIGINT`** : le gap lock porte sur elle — transposition
**réellement** exacte du pattern éprouvé —, et `code` en est la **projection** base 26
bijective, calculée en Rust. ⚠️ **La conversion est une fonction pure : elle se teste dans
`kesh-core`, sans base**, et T1 nomme ses cas limites (`26 → Z`, `27 → AA`, `53 → BA`,
`702 → ZZ`, `703 → AAA`). Les deux contraintes d'unicité, sur `seq` et sur `code`, tiennent
le filet.

**P5-2 (HIGH) — le DDL était un fragment.** Il portait un `...` et laissait au développeur les
FK, les `ON DELETE`, le `CHARACTER SET`/`COLLATE`, les `COMMENT` et l'`ENGINE` — alors que les
migrations du dépôt les déclarent toutes explicitement (comparé à
`20260814000001_contacts_client_number_canonical.sql`). Le DDL est désormais **entier et à
reprendre tel quel**.

**P5-3 (MEDIUM) — les comportements de FK sont tranchés**, conformes aux conventions relevées
au sol (`companies(id) ON DELETE RESTRICT` partout ; `users(id) RESTRICT` pour un auteur
d'action, cf. `bank_imports.imported_by_user_id`). ⚠️ Le choix de **`RESTRICT`** sur
`journal_entry_lines.lettering_id` impose l'ordre du délettrage — mettre les lignes à `NULL`
**puis** supprimer l'entrée — et c'est voulu : un `SET NULL` ferait disparaître une marque des
deux côtés **sans trace**, le défaut muet même qu'AC8 ferme ailleurs.

**P5-4 (MEDIUM)** : T7 énumérait les critères sans dire qu'AC4 porte **deux** causes
distinctes — huit messages, pas sept. **P5-5 (LOW)** : T1 précise que le compteur du test P6
bougera, mais qu'aucun backfill à fenêtre n'est en jeu.

**Ce que cette passe apprend sur le processus** : un arbitrage du Project Lead **n'est pas
exempt de revue**. Celui-ci était juste dans ses deux choix — la table et le format restent
les bons — mais sa **traduction en DDL** portait un défaut critique que ni l'arbitrage ni les
quatre passes précédentes ne pouvaient contenir, puisqu'il n'existait pas encore. Une passe
6 est due.

### Arbitrage du Project Lead — 2026-08-25

Les deux décisions que les passes avaient laissées ouvertes sont **tranchées par Guy**, et
inscrites au § *Décisions tranchées*.

**Le porteur : une table `letterings`.** Elle transpose à l'identique le gap lock éprouvé
d'`entry_number` — sa propre `company_id` le permet —, garde un **filet au niveau du schéma**
(`uq_letterings_company_code`) si le verrouillage faiblit, et fournit `created_at` /
`created_by` dont AC13 a besoin. La colonne sur `journal_entry_lines` n'offrait aucun des
trois : pas de `company_id` à verrouiller, aucune contrainte possible — une contrainte
d'unicité y aurait même **interdit AC1** — et nulle part où porter l'auteur.

**Le format : une séquence alphabétique par société**, `A`, `B`, … `Z`, `AA` — base 26
bijective. C'est le format des logiciels comptables suisses, Bexio compris, dont **D6 fait
déjà le modèle de l'écran** ; il est court, lisible à l'œil sur une ligne, et **se dicte au
téléphone**. Un identifiant sans compteur (ULID, UUID) supprimerait la contention mais 15-1c
exige que la marque soit **visible sur la ligne** : vingt-six caractères y sont
inutilisables.

**Propagé dans le même patch** : AC3 (la génération sous gap lock, le filet de schéma, et un
**test de concurrence désormais exigé** — sans lui le filet reste une intention), T1 (le DDL
est connu, `CREATE TABLE` + FK nullable), T2 (l'ajout de `letterings` à `reset_demo` devient
**obligatoire** — sa liste de `DELETE` est explicite, et le bloc s'exécute sous
`FOREIGN_KEY_CHECKS=0`), et le triage P7 (ni `CREATE TABLE` ni `ADD COLUMN` n'entre dans le
détecteur).

⚠️ **Une passe de contrôle reste due.** Cet arbitrage fixe une règle structurante **après**
la convergence de la passe 4 : la passe ciblée à une lentille codifiée ce matin
(§ A9, PR #355) **ne s'applique pas** — sa borne exclut les patches qui changent une règle
métier.

### Passe 4 de `validate` — 2026-08-25 (Sonnet, contexte frais)

✅ **CONVERGENCE. Un seul finding, de sévérité LOW** — le critère d'arrêt de la § *Review
Iteration Rule* est atteint (« uniquement des findings de sévérité `LOW` »).

**Trajectoire complète** : `2 HIGH` → `2 MED` → `1 CRIT + 2 HIGH` → **`1 LOW`**.

**Ce que la passe a vérifié en premier, et c'était sa raison d'être** : la correction du
CRITICAL par la passe 3 est-elle elle-même juste ? Elle a relu `update` **en entier**
(l. 782-1123) plutôt que les extraits cités, et conclut que **la clause (ii) d'AC7 est
implémentable sans conflit** :

- le `UPDATE` de l'en-tête, le `version + 1` et le snapshot d'audit restent valides —
  `entry_snapshot_json` relit les lignes après coup, et des lignes non touchées se relisent
  correctement, mêmes `id` et même `line_order` ;
- ⚠️ **`is_no_op_change` doit rester ENTIÈRE pour le court-circuit KF-004** ; c'est une
  **seconde** fonction, tirée de sa moitié « lignes », qui pilote la clause (ii). Aucun
  risque de régression sur KF-004 ;
- le comparateur de lignes est **exhaustif** vis-à-vis de `NewJournalEntryLine`, dont les
  champs sont exactement `account_id`, `debit`, `credit`, `project_id` — rien n'y échappe. Un
  changement du **nombre** de lignes est classé « changé » avant toute comparaison
  positionnelle ;
- ⚠️ **aucun chemin ne fait survivre une marque à une modification qui aurait dû la
  détruire.** Toute modification touchant une ligne bascule sous la clause (i).

**Une conséquence assumée, relevée par la passe et vraie** : modifier une ligne **non
lettrée** dans une écriture qui en compte trois dont **une seule** est lettrée est refusé
aussi. C'est la conséquence directe du mode d'écriture par `DELETE`/`INSERT` global — pas un
défaut, mais une restriction à connaître.

**P4-1 (LOW)** — la garde de clôture était citée `journal_entries.rs:892` ; le
`return Err(DbError::FiscalYearClosed)` est en **894**, le test en 892. La citation porte
désormais la garde entière, `892-894`. **Toutes les autres citations de la passe 3 sont
vérifiées exactes** (`:981`, `:782`, `:1226`, `:232`, `invoices.rs:1338`, `:1235`).

**Contrôles et réfutations propres à cette passe** : les deux seuls appelants de
`delete_in_tx` dans tout le workspace sont confirmés (la route et `invoices::delete`) ; AC12
est sans ambiguïté de devise ni d'arrondi — pas de colonne de devise, `Decimal` exact, débit
et crédit mutuellement exclusifs par contrainte ; AC13 se conforme au motif `entité.verbe` du
dépôt, et la question de l'`entity_id` d'un événement portant sur **deux** lignes **a été
posée puis refermée** par D1 elle-même — la marque portant sur la ligne, l'entité d'audit est
la ligne, un événement par ligne, que le porteur soit (A) ou (B). Enfin les compteurs de
`docs/migrations-idempotence-audit.md` ont été recomptés depuis la source : **61 partout**,
aucune dérive préexistante que T1 hériterait.

**Verdict : la spécification est prête pour le développement**, une fois arbitrées les deux
décisions réservées au Project Lead — le **porteur** de la marque (A ou B) et son **format**.

### Passe 3 de `validate` — 2026-08-25 (Opus, contexte frais)

⛔ **1 CRITICAL, 2 HIGH, 4 MEDIUM, 2 LOW. Trajectoire : `HIGH → MEDIUM → CRITICAL`.**

⚠️ **Le critère de non-convergence est déclenché une seconde fois — mais il ne commande PAS
un re-split ici, et la raison importe.** 15-1a est **étroite** : ce n'est pas un défaut de
largeur. C'est que **la passe 2 a tranché une règle métier sur un fait qu'elle n'avait pas
vérifié**, et que la conduite retenue rouvrait le trou que la story existe pour fermer. Le
remède est ciblé.

**P3-1 (CRITICAL) — la conduite tranchée en passe 2 détruisait la marque sur le chemin
qu'elle ouvrait.** AC7 autorisait la modification de l'en-tête. Or le `DELETE FROM
journal_entry_lines` de `update` (`journal_entries.rs:981`) n'est gardé que par le
court-circuit **no-op complet** — en-tête **et** lignes identiques. Une modification de
libellé seul le franchit **systématiquement** : les lignes sont effacées et réinsérées sans
marque, la contrepartie garde la sienne. **Mot pour mot le mode d'échec que cette story
existe pour fermer**, réintroduit par le critère censé le fermer.

Et le lien n'avait jamais été fait : l'argument (2) de la passe 2 écartait la préservation de
la marque comme « pas fiable », alors qu'autoriser l'en-tête *l'exigeait* dans ce cas précis.
→ **AC7 clause (ii)** : quand les lignes sont inchangées, `update` **ne les touche pas**. La
condition de la garde est exactement ce qui rend le `DELETE`/`INSERT` inutile — le même patch
ferme le CRITICAL et vide l'objection de son objet.

**P3-2 (HIGH) — le premier des « deux faits au sol » de la passe 2 était FAUX.** Elle
affirmait qu'un refus global rendrait une écriture lettrée sur exercice clos « figée jusqu'à
sa description ». **Elle l'est déjà** : `update` refuse à l'Étape 2
(`journal_entries.rs:892-894`, `DbError::FiscalYearClosed`), lettrage ou non. Le cas réel est
plus étroit — écriture sur exercice **ouvert** lettrée contre une contrepartie sur exercice
**clos**. ⚠️ Un développeur écrivant le test d'AC7 depuis ce texte aurait obtenu un 409
inattendu, et aurait pu conclure que **la garde de clôture est l'obstacle et l'assouplir** :
une régression sur l'immuabilité post-clôture introduite par un patch de lettrage.

**P3-3 (HIGH) — le split avait laissé tomber l'égalité des montants.** La garde vivait dans
**15-1c**, la story de l'écran, alors que **la route d'écriture est dans 15-1a**. 15-1a et
15-1b mergées sans 15-1c, un appel direct appariait 1000 avec 300 et la vue retirait la
créance en laissant 700 dus — le finding **F2** de la story mère, coté HIGH deux fois,
survivant au découpage. Le tableau des décisions du split assignait à 15-1c « la **tolérance**
de montant » : la tolérance et l'existence du contrôle sont deux questions, et le split a
déplacé la seconde en croyant ne déplacer que la première. → **AC12**.

**Quatre MEDIUM** : **P3-4** T7 annonçait « trois messages », il en manquait **quatre** — et
la garde i18n ne rattrape pas une clé *jamais écrite* ; **P3-5** le **format** de la marque
n'était fixé nulle part alors que T1 doit écrire le DDL et que P8 interdit d'y revenir — la
story mère nommait pourtant le trou dans ses deux dimensions, « ni format, ni portée », et le
split a **perdu le format** ; **P3-6** la réfutation du wipe `reset_demo` ne vaut que pour la
conduite (A) — une table `letterings` n'est pas dans sa liste explicite de `DELETE` ;
**P3-7** aucune trace d'audit n'était exigée, alors qu'AC5 autorise une écriture sur exercice
**clos**, seul endroit où l'absence de trace est grave. → **AC13**.

**Deux LOW** : chemins de fichier imprécis (P3-8) ; et **P3-9**, qui compte plus que sa
cote — « sa seconde moitié EST la garde » invitait à réutiliser `is_no_op_change` **entière**,
laquelle retourne `false` dès que l'en-tête diffère **sans regarder les lignes**, soit
l'exact contraire de la conduite. S'y ajoute une réserve sur `project_id`, dont le
comparateur n'est exact que parce que les deux sites de la route passent `None`.

**Pistes réfutées, et l'une répondait à une inquiétude explicite** : ⚠️ **changer la DATE ne
peut pas déplacer une écriture lettrée d'exercice** — `update` ne réécrit jamais
`fiscal_year_id`, et confine `entry_date` aux bornes de l'exercice de l'écriture. La garantie
d'AC5/AC6 survit aux modifications d'en-tête. Changer le **journal** est inerte. Les
`DELETE FROM journal_entries` d'`invoices.rs:3866` et l'`INSERT` de `journal_entries.rs:2297`
sont dans des `mod tests`. Le point de passage unique d'AC8 tient. Le lettrage d'une ligne
avec elle-même est déjà fermé par la contrainte `chk_jel_debit_credit_exclusive`. Et **les
patches de la passe 1 tiennent sans régression** — AC10 et AC11 purement additifs, numéros de
ligne exacts.

La spec passe de **11 à 13 critères** *(rectifié en passe 8 : « 14 » avait été écrit sans recompter — `grep -c "^\*\*AC[0-9]*\*\*"` rend **13**, AC1 à AC13 sans trou. C'est exactement le geste que la § *Recompter ses propres comptes rendus* vise, commis dans un fichier qui l'invoque)*. **Verdict : passe 4 due**, modèle différent.

### Passe 2 de `validate` — 2026-08-25 (Haiku, contexte frais)

**0 HIGH, 2 MEDIUM, 1 LOW** — et les trois portent sur **le même point**. La sévérité
décroît (`HIGH → MEDIUM`) : convergence monotone, le critère de split n'est pas déclenché.

**Aucune régression des patches de la passe 1** : AC10 et AC11 sont jugés cohérents et
testables, et le bloc sur l'asymétrie (A)/(B) exact au sol.

⚠️ **Le finding, et il est embarrassant : AC7 disait « à trancher à la spécification et non à
l'implémentation », puis ne tranchait pas.** C'est **mot pour mot** le défaut relevé sur AC12
de la story mère en passe 2 — reproduit par celui-là même qui l'avait relevé, dans la story
née de ce relevé. Les deux MEDIUM et le LOW en découlent en cascade : le décompte des messages
i18n de T7 était indéterminé, et le sort d'une renumérotation restait ouvert.

**AC7 est tranchée, et deux faits au sol l'ont tranchée — pas une préférence :**

> La modification est **refusée si elle change les LIGNES** d'une écriture dont une ligne est
> lettrée. **L'en-tête reste modifiable.**

**(1) Un refus global figerait l'écriture jusqu'à son libellé.** `update` réécrit
systématiquement toutes les lignes dès que le payload diffère — aucun chemin « en-tête
seul » n'existe. Refuser en bloc obligerait à délettrer pour corriger une faute de frappe,
**or le délettrage est interdit sur exercice clos (AC6)** : une écriture lettrée sur un
exercice clos deviendrait définitivement figée, description comprise. Cet effet de bord
n'avait été vu par aucune des deux conduites que la spec proposait.

**(2) Préserver la marque à travers le `DELETE`/`INSERT` n'est pas fiable** : la réinsertion
numérote par position, rien ne rattache une ligne nouvelle à l'ancienne si l'ordre change ou
si une ligne est insérée. La marque atterrirait sur la mauvaise ligne — **pire qu'une marque
perdue, puisque plausible**.

⚠️ **La conduite retenue n'était dans aucune des deux options offertes** — c'est une
troisième, trouvée en cherchant *pourquoi* les deux premières coûtaient cher. Et elle
n'invente rien : `is_no_op_change` (`journal_entries.rs:782`) compare déjà l'en-tête **puis**
les lignes ; **sa seconde moitié EST la garde demandée**. Elle règle aussi la renumérotation
(P2-3) sans clause dédiée : réordonner *est* un changement de lignes pour un comparateur
positionnel.

La spec reste à **11 critères**. **Verdict : passe 3 due** — deux MEDIUM au rapport.

### Passe 1 de `validate` — 2026-08-25 (Sonnet, contexte frais)

**2 HIGH, 1 MEDIUM, 1 LOW.** Tous vérifiés au sol par l'orchestrateur avant application.

⚠️ **Et un résultat POSITIF, qui vaut d'être écrit** : la passe a **refait le recensement des
chemins d'écriture en indépendant** — `grep -rn "INSERT INTO journal_entry_lines\|DELETE FROM
journal_entry_lines\|UPDATE journal_entry_lines" crates/` — et conclut qu'il est **exhaustif** :
`create`, `update` et le wipe de `reset_demo`, rien d'autre. Elle a aussi vérifié que
`update`/`delete_by_id`/`delete_in_tx` n'ont **aucun appelant** hors les deux routes et
`invoices::delete` — la réconciliation, les avoirs et le règlement fournisseur créent tous des
écritures **nouvelles** via `create_in_tx`, jamais de modification. **Le socle de raisonnement
de cette story est confirmé, pas seulement non réfuté.**

| | défaut | gravité | remède |
|---|---|---|---|
| **PA-1** | **Aucun critère ne refusait de lettrer une ligne DÉJÀ lettrée.** Ré-apparier écrase la marque et laisse l'ancien partenaire **seul avec l'ancienne** — réputé soldé à vie, sans contrepartie, et rien ne le signale. AC4 laissait passer le cas, compte et sens étant corrects | **HIGH** | **AC10** |
| **PA-2** | **(A) et (B) n'offrent pas la même garantie sur AC3**, et la spec ne le disait pas. Le seul pattern éprouvé du dépôt (gap lock d'`entry_number`, `journal_entries.rs:232`) verrouille une `company_id` que `journal_entry_lines` **n'a pas** : (B) le transpose et garde un filet `UNIQUE`, (A) doit verrouiller par jointure **sans aucun filet de schéma** | **HIGH** | l'asymétrie est nommée au § *Décision à trancher* — c'est un **input d'arbitrage**, pas un blocage |
| **PA-3** | **Aucun critère n'exigeait que les deux lignes appartiennent à la société de l'APPELANT.** « Même compte » les lie entre elles, pas à l'appelant : c'est un **IDOR**, et le dépôt en a déjà payé un (KF-002) | MEDIUM | **AC11** |
| **PA-4** | T1 citait P1, P5 et P7 mais **pas P6** | LOW | le grep de recensement ajouté à T1 |

⚠️ **PA-1 est le finding qui compte, et son intérêt dépasse la story** : les passes
précédentes avaient fermé les chemins par lesquels un **autre** dispositif détruit la marque
(modification, suppression). Celui-ci est une porte que **le lettrage ouvre lui-même** — le
défaut muet vient de la fonctionnalité, pas de son environnement. Aucune des quatre passes de
la story mère ne l'avait vu.

**Pistes réfutées** : le wipe de `reset_demo` (il efface tout ensemble, cohérent) ; le
`.keshbackup` (ses colonnes viennent d'`information_schema`) ; l'inutilité de l'exemption P7
(confirmée — `writes_data()` ne matche que le premier mot-clé, `ADD COLUMN` n'y entre jamais) ;
`fiscal_years::close`/`reopen`, lus en entier, ne touchent **jamais** `journal_entry_lines`.
Et **aucune troisième conduite** ne manque au § *Décision à trancher* : les deux présentées
sont faisables, un `UPDATE`-par-id supposerait de faire porter un `id` par
`NewJournalEntryLine`, changement de contrat qui déborde le socle.

La spec passe de **9 à 11 critères**. **Verdict : passe 2 due** — deux HIGH et un MEDIUM au
rapport.

### Création par split de la 15-1 — 2026-08-25

Issue du **split de la Story 15-1**, décidé par le Project Lead après que la passe 3 de
`validate` eut déclenché le critère de non-convergence (`MEDIUM → HIGH`). Cette story
recueille les deux HIGH de cycle de vie (**P3-1**, **P3-2**), le MEDIUM d'unicité
(**P3-5**) et deux LOW (**P3-9** export CSV, **P3-10** exemption P7 inutile).

⚠️ **Une décision reste ouverte et bloque le développement** : le porteur de la marque
(colonne ou table `letterings`). Elle est posée au § *Décision à trancher*, avec les deux
conduites et ce que chacune coûte.
