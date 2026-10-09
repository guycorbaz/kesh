# Story 15.1a2 : Le lettrage des pièces — index (découpée)

## Status

split

⛔ **CORPS VIDÉ — cette fiche ne contient plus ni décisions, ni critères, ni tâches** *(découpée le
2026-10-09 à la remédiation de sa validation P1, C-15-1a2-1)*. Elle garde les pointeurs vers ses deux
sous-fiches, la table qui dit où chaque élément est allé, l'intégration des sections « Reçu », le
recompte aux deux bornes, le bilan de la validation P1 et l'historique (Change Log). Le corps d'avant
le découpage se lit dans l'historique git (`git show 056997b0:_bmad-output/implementation-artifacts/15-1a2-lettrage-des-pieces.md`).
*(Précédents : 15-1, 15-12, 15-1a — C124.)* **La numérotation est conservée** dans les sous-fiches :
les renvois des fiches sœurs (« 15-1a2 P2 », « AC5 de la 15-1a2 », « 15-1a2 P5 », « point 9 de son
Reçu ») restent justes ; les numéros neufs commencent à P7 et AC13.

## Les trois sous-fiches

| ordre | fiche | ce qu'elle porte |
|---|---|---|
| 0 | `15-1a2-0-lettrage-fige-avec-la-periode.md` | **Le lettrage se fige avec la période** *(créée à la validation P3, C-15-1a2-19)* : la règle des périodes, une factorisation publique (`open_period_rule`, D1) ; le **refus** du geste qui délettrerait un groupe `document` entièrement clos — rang 2 bis de la file commune (`DocumentLetteringInClosedPeriods`, `document_group_frozen_by_periods`), refusé par les **quatre** gestes d'annulation (règlement et solde clients, dé-rapprochement, paiement et facture fournisseurs), code réemployé, textes ×4 locales, écran, remède précis, documentation — ; le message `LETTERING_IS_DOCUMENT` neutre. **Dormant** jusqu'à la 15-1a2-i : éprouvé sur des groupes posés en SQL brut |
| 1 | `15-1a2-i-lettrage-des-pieces-clients.md` | **Les pièces clientes** : le groupe `document` d'une facture (P1), la synchronisation et sa jumelle de dissolution (P3), sur la règle des périodes de la 15-1a2-0 — abstention à la création ; le refus, livré par la 15-1a2-0, y est intégré (AC14) —, `DocumentRef` (défini en P3) et l'extension d'audit, les cinq sites — `settle_invoice`, `write_off_invoice`, `accept_one_invoice` (et son mappage per-proposal), `create_credit_note`, `cancel_settlement_in_tx` client qui couvre le dé-rapprochement (P4, P5), le compte de créance non lettrable, la documentation client |
| 2 | `15-1a2-ii-fournisseurs-et-rattrapage.md` | **Les fournisseurs et le rattrapage** : le groupe `document` d'une facture fournisseur et sa découverte par statut (P2), sa synchronisation (P3 part ii ; le rang 2 bis de ses deux annulations est à la 15-1a2-0), les trois sites — `pay_in_tx` (qui couvre `pay` et `confirm_batch`, y compris le compte interne), `cancel_settlement_in_tx` et `cancel_in_tx` fournisseurs (P4 part ii) —, les **deux** migrations de rattrapage (P6), leur triage P7, l'outillage P5/P6/P8, le rejeu après restauration, la documentation fournisseur et administrateur |

**Dépendances** : la **15-1a2-i suppose la 15-1a2-0 mergée** (règle des périodes, refus déjà en place
quand la dissolution arrive) ; la **15-1a2-ii suppose les deux** (synchronisation factorisée, audit,
fixture ; son AC6 compare le rattrapage à la synchronisation de la 15-1a2-i). Ordre :
**15-12a → 15-12b → 15-1a-i → 15-1a-ii → 15-1a2-0 → 15-1a2-i → 15-1a2-ii → 15-1b-0 → 15-1b → 15-1c**. ⛔ Pas de tag entre les
merges (C124) : entre la 15-1a2-i et la 15-1a2-ii, les factures fournisseurs payées ne sont pas
lettrées et les données d'avant la mise à jour ne sont pas rattrapées.

⚠️ **`confirm_batch` est fournisseur** : il appelle `supplier_invoices::pay_in_tx`
(`payment_batches.rs:456`). La consigne de découpage le rangeait parmi les règlements clients ; il est
dans la **15-1a2-ii**, avec `pay_in_tx`, qui le couvre.

## Pourquoi le découpage

Finding **F-9** (lentille F, validation P1) : la fiche touchait au moins **sept** modules de premier
niveau — `letterings.rs`, `invoice_settlements(_write).rs`, `credit_notes.rs`, `supplier_invoices.rs`,
`post_restore.rs` et les migrations, `kesh-api` `routes/reconciliation.rs` (la signature de
`create_in_tx` ne permettait pas l'appel caché que supposait la fiche, R1/F-5), le message i18n et la
documentation — pour un seuil de **cinq** (`CLAUDE.md`, « Règle de splitting préventif », premier
critère ; le second, la non-convergence, ne joue pas en première passe). Décision de l'orchestrateur,
**C-15-1a2-1** : couture *pièces clientes* / *fournisseurs et rattrapage*. Chaque sous-fiche compte
**cinq** modules (comptage écrit dans ses Dev Notes).

**Second découpage, validation P3** (finding **F-1** de la 15-1a2-i, lentille F) : le refus du rang 2 bis,
ajouté à la P2, avait fait passer la 15-1a2-i à six domaines sous une dérogation (C-15-1a2-13) qui
n'examinait qu'**un** ordre de merge — refus **après** la synchronisation. L'ordre inverse — **le refus
d'abord, dormant**, éprouvé sur des groupes posés en SQL brut — ne laisse aucun état défaillant. Décision de
l'orchestrateur, **C-15-1a2-19** : la story préalable **15-1a2-0** ; dérogation **retirée**. Les modules se
comptent désormais aux **deux** grains, comme à la 15-1a-i (C-15-1a2-21) : 15-1a2-0 — 4 crates/paquets,
**11** modules métier (dont 7 mécaniques ; signal déclaré) ; 15-1a2-i — 2 et **5** ; 15-1a2-ii — 1 et **4**.

## Table de correspondance — où chaque élément est allé

| élément de la 15-1a2 | 15-1a2-i | 15-1a2-ii |
|---|---|---|
| *(Validation P3 : la règle des périodes, le refus du rang 2 bis et ses refus fournisseurs, ses textes et son écran, le message `LETTERING_IS_DOCUMENT` sont partis à la **15-1a2-0** — D1 à D5, AC1 à AC9 ; les lignes ci-dessous le disent.)* | | |
| Story, « Pourquoi », modèle client | ✓ | — |
| Modèle fournisseur | — | ✓ |
| P1 (groupe client) | ✓ (+ lettrabilité, période) | renvoi |
| P2 (groupe fournisseur) | — | ✓ |
| P3 (synchronisation) | ✓ (client, algorithme, audit) | part ii (fournisseur, factorisée) |
| P4 (sites) | 5 sites client | 3 sites fournisseur |
| P5 (dissolutions, exercice clos) | ✓ (réécrite, F-12) | renvoi |
| P6 (rattrapage) | — | ✓ (deux migrations, C-15-1a2-4) |
| **P7** (périodes closes) — neuf | ✓ (abstention ; règle et refus : renvoi à la 15-1a2-0) | renvoi |
| AC1–AC5 | ✓ (AC3, AC5 réécrits — 15-6a mergée) | — |
| AC6 (accord rattrapage ↔ sync) | — | ✓ |
| AC7 (fournisseur) | — | ✓ (+ `confirm_batch`, compte interne, `B` non lettrable) |
| AC8 (inventaire lexical) | part i (règlements, avoir, annulation client) | part ii (`settlement_journal_entry_id`) |
| AC9 (invariant) | part i | part ii |
| AC10 (audit) | part i | part ii |
| AC11 (migration) | — | ✓ |
| AC12 (documentation) | part i | part ii |
| **AC13** (créance non lettrable), **AC14** (périodes closes : intégration ; l'ancien (d) → 15-1a2-0 AC6), **AC15** (rapprochement) — neufs | ✓ | — |
| **AC17** (le refus du rang 2 bis, du prédicteur à l'écran) — neuf, validation P2 | → **15-1a2-0** (AC2–AC7), numéro non réattribué | — |
| **AC16** (rejeu après restauration) — neuf | — | ✓ |
| T1–T6 | T0–T6 (client) | T0–T6 (fournisseur, migrations) |

## Les sections « Reçu » — où chaque point est intégré

| point | objet | intégré à |
|---|---|---|
| Reçu 15-1a, 1 et 13 | `min_required` déjà à 0.13.0, motif exact | 15-1a2-ii P6 (« pas de relèvement ») |
| 2, 11, 15 | exercice tenu par site, verrou un par un | P4 des deux sous-fiches (colonne « exercice tenu » : les verrous **existants** des gestes, aucun neuf) |
| 3, 10 | rattrapage des groupes clos ou sous la borne | 15-1a2-ii P6 (abstention, C-15-1a2-2) |
| 4 (C104) | lettrabilité à la création seule | 15-1a2-i P3 point 3, AC13 |
| 5 (C106) | paire `reversal` avec ligne de pièce | 15-1a2-ii P6 (M1 étape 3, classe A) |
| 6, 19 | facture créditée **et** réglée | 15-1a2-i P1 — **tranché** : reste ouverte, aucun site réécrit (C-15-1a2-7) |
| 7 | appariement par position | 15-1a2-ii P6 |
| 8, 14 | ordre de développement | Status des sous-fiches, ci-dessus |
| 9 | annulation d'un règlement de période verrouillée | 15-1a2-0 D2-D3 (ex-15-1a2-i P7 point 2) — **refusée** (rang 2 bis, C-15-1a2-10, C-15-1a2-19 ; l'abstention et le groupe gardé de la P1, C-15-1a2-2, sont révisés) |
| 12 (C116) | marques rendues par la contre-passation | **sans objet** : aucune synchronisation n'est appelée après une contre-passation (C-15-1a2-9) |
| 16 (C126) | `ENTRY_LETTERED` parle en dernier | **sans objet** : aucun refus n'est ajouté à `delete_in_tx` ni à `invoices::unvalidate` |
| 17, 18 (C127, C128) | exercice par ligne dans l'audit, nom lu sans verrou | 15-1a2-i AC10, P3 (aucune lecture verrouillante ajoutée) |
| 20 (C129) | « reste intacte » | 15-1a2-i AC12 (avoirs) |
| Reçu 15-1a-i (textes provisoires) | état réel au HEAD : `CHANGELOG.md:15`, `api-external.md:223`, `:291`, `:324`, glossaire `user-manual.tex:2420-2426` **coupé sur deux lignes**, et `:765` (absent du Reçu) | 15-1a2-i AC12 (client) ; 15-1a2-0 AC9 (message `LETTERING_IS_DOCUMENT`) ; 15-1a2-ii AC12 (fournisseur) |

## Recompte aux deux bornes

*(Borne basse : la fiche au commit `056997b0`, corps avant `## Change Log`. Borne haute : les trois
sous-fiches au commit de la remédiation de la validation P3. Commandes : `grep -cE '^\*\*AC[0-9]+'` et
`grep -cE '^- \[ \] \*\*T'` sur chaque fichier.)*

| | 15-1a2 (`056997b0`) | 15-1a2-0 | 15-1a2-i | 15-1a2-ii |
|---|---|---|---|---|
| critères | 12 (AC1–AC12) | 9 (AC1–AC9, numérotation propre) | 12 (AC1–AC5, AC8–AC10, AC12–AC15 ; AC17 déplacé, numéro non réattribué) | 8 (AC6, AC7, AC8–AC10, AC11, AC12, AC16) |
| tâches | 6 (T1–T6) | 8 (T0–T7) | 7 (T0–T6) | 7 (T0–T6) |
| tests nommés | aucun | 12 neufs + 4 modifiés | 27 neufs + 1 étendu | 18 neufs + 5 modifiés |

Critères distincts de la numérotation conservée : AC1–AC16 (16) ; AC8, AC9, AC10, AC12 se partagent en parts
i / ii (12 + 8 − 4 = 16) ; s'y ajoutent les 9 critères propres de la 15-1a2-0.
*(Recompté après la remédiation de la validation P3 ; à la P2 : 13 / 8 / 30 + 3 pour la part i, 8 / 7 / 19 +
4 pour la part ii ; à la P1 : 12 / 7 / 22 + 1 et 8 / 7 / 14 + 2.)*

## Bilan de la validation P1 — par finding

Rapports : `/home/gcorbaz/devel/kesh-gate-logs/15-1a2-validate-p1-R.md` (Sonnet 5.5, Auditeur
d'acceptation : **3 HIGH, 7 MEDIUM, 4 LOW**) et `…-F.md` (Sonnet 5.5, adversaire plein champ : **0
CRITICAL, 2 HIGH, 7 MEDIUM, 3 LOW**). Chaque affirmation relue au code (`grep -nF` sur `056997b0`).

| finding | sévérité | verdict | où |
|---|---|---|---|
| R1 ≈ F-5 — signatures sans exercice tenu ni acteur ; `create_in_tx` ne « couvre » rien ; audit sans couture ; `accept_one_invoice` sans mappage | HIGH / MEDIUM | **corrigé** (appels explicites, exercice et acteur par site, `*_inner` + `DocumentRef`, mappage per-proposal) | i P3, P4, AC10, AC15 ; ii P3, P4 |
| R2 = F-6 — périodes closes non tranchées, AC5/AC6/15-1b D1 contradictoires ; quatre décisions « à trancher » | HIGH / MEDIUM | **corrigé** (abstention des deux côtés, C-15-1a2-2 ; AC5 borné ; AC14 ; « Pour la 15-1b ») | i P7, AC5, AC14 ; ii P6, AC6 |
| R3 = F-1 — paires `reversal` en classe A réécrivent un délettrage voulu | HIGH | **corrigé** (M1 au registre, M2 exemptée ; paires d'achats annulés en classe A, non dissolubles à la main) | ii P6, AC11, AC16 |
| F-2 = R5 — compte non lettrable : échec du règlement ; SQL et Rust divergent | HIGH / MEDIUM | **corrigé** (sauté sans erreur, prédicat recopié en SQL, C-15-1a2-3) | i P3, AC13 ; ii P3, P6, AC7 |
| R4 — Reçus non intégrés | MEDIUM | **corrigé** | table ci-dessus |
| R6 — couture des détails d'audit, clé d'API | MEDIUM | **corrigé** (écart `api_key_id: None` nommé, sauf rapprochement) | i AC10 |
| R7 = F-4 — avoir synchronisé avant d'exister | MEDIUM | **corrigé** (après la bascule `cancelled`) | i P4 |
| R8 = F-8 — 15-6a–d mergées ; ancres périmées | MEDIUM | **corrigé** (avertissements #473/#474 retirés, AC3/AC5 réécrits, ancres sur `056997b0` citées par fonction) | i, ii « Le modèle réel » |
| R9 = F-7 — manuel, PDF, glossaire coupé, `:765`, avoirs, message `LETTERING_IS_DOCUMENT` | MEDIUM | **corrigé** | i AC12, ii AC12 |
| R10 = F-13 — outillage de migration | MEDIUM / LOW | **corrigé** ; ⚠️ R10 proposait « squash inchangé » : **réfuté** sur ce point — `test-schema/README.md` dit que le garde compare aussi le suivi `_sqlx_migrations`, d'où une régénération sans diff de structure (F-13) ; compteurs 76 → **78** (deux migrations) | ii AC11 |
| R11 — lexical : exclusion des tests, corps de fonction, avoir oublié | MEDIUM | **corrigé** (réutilise `decouper` / `bloc_apres` / `blocs_de_test`, `INSERT INTO credit_notes` inclus) | i AC8, ii AC8 |
| F-3 — cycle d'`accept_one_invoice`, lecture non verrouillante, `LetteringLineAlreadyLettered` mal mappé | MEDIUM | **corrigé** (synchronisation **après** (g) : même ordre que les autres gestes ; découverte verrouillante ; mappage) | i P3, P4, AC15 |
| F-9 — découpage | MEDIUM | **corrigé** (découpée, C-15-1a2-1) | cet index |
| F-10 — tests existants, `confirm_batch`, compte interne | MEDIUM | **corrigé** | i Dev Notes ; ii AC7, Dev Notes |
| R12 — facture à TTC nul | LOW | **réfuté** : inatteignable (`EntryZeroTotal`, `balance.rs:196`) — écrit au modèle | i « Le modèle réel » |
| R13 — « groupe existant » ambigu ; phrase de P5 | LOW | **corrigé** | i P3 étape 2, P5 |
| R14 — analyse des verrous antérieure au socle | LOW | **corrigé** (cycles nommés, rejeu comme défense, routes `Rejouee` listées) | i Dev Notes, T0 ; ii T0 |
| F-11 — dissolution sans groupe | LOW | **corrigé** (no-op) | i P3, AC4 |
| F-12 — phrase inachevée de P5 | LOW | **corrigé** | i P5 |

**Décisions de l'orchestrateur appliquées, et ce qui s'y est heurté** :

1. **Découpage** — appliqué ; `confirm_batch` déplacé côté fournisseur (voir plus haut).
2. **Périodes closes** — appliqué. *(⚠️ Révisé à la validation P2 : le côté « délettrer » est remplacé par
   un **refus**, C-15-1a2-10 ; ce qui suit décrit la décision P1.)* ⚠️ **Heurt** : « la pièce reste ouverte, **lettrable à la main** si la
   règle le permet » est **faux** — une ligne de pièce n'est jamais lettrable à la main (15-1a R5, rang 5
   de `create_group_in_tx` en mode `Manual`, `LETTERING_LINE_OWNED_BY_DOCUMENT`). Écrit tel quel :
   ouverte, **non** lettrable à la main. ⚠️ **Coût du côté « délettrer »**, écrit et non réparé : un
   groupe gardé après l'annulation d'un règlement de période verrouillée laisse une facture **due** dont
   la ligne de vente est **lettrée** (exception d'AC5 et d'AC9), et un nouveau règlement de cette facture
   reste ouvert face au miroir (15-1a2-i P7 points 2-3). À rouvrir si la recette le juge trop cher.
3. **Compte non lettrable** — appliqué tel quel.
4. **Paires `reversal`** — appliqué, avec deux précisions imposées par le code : (a) **deux** fichiers de
   migration (M1 au registre, M2 exemptée) — le registre exige qu'un extrait porte toutes les écritures
   de sa migration, et une migration ne peut être à la fois au registre et exemptée ; les compteurs P5
   passent donc de 76 à **78**, non 77 ; (b) les paires dont l'origine est l'**achat** d'une facture
   fournisseur annulée vont dans M1 (classe A) : elles ne se dissolvent pas à la main (C106), un `NULL` n'y
   est donc pas un choix — et, exemptées, elles resteraient ouvertes **et** non lettrables à la main
   après restauration d'une sauvegarde ancienne.
5. **Le reste** — appliqué.

## Change Log

### Validation P3 — 2026-10-09 (Sonnet 5.5 ×2 par sous-fiche ; remédiation Opus 5.5, seul remédiateur des fiches de la suite du lettrage)

Bilans par finding dans les Change Logs des sous-fiches. 15-1a2-i : **4 MEDIUM distincts**, trois nés de la
remédiation P2 (signal D5 levé) ; 15-1a2-ii : **2 MEDIUM distincts**. Décision structurante de l'orchestrateur
(**C-15-1a2-19**, finding F-1) : le refus du rang 2 bis devient la story préalable **15-1a2-0**, dormante, éprouvée
en SQL brut — **trois** sous-fiches désormais ; dérogation C-15-1a2-13 **retirée** ; modules comptés aux deux grains
(C-15-1a2-21). Corps de cet index : sous-fiches, dépendances, « Pourquoi le découpage », table de correspondance,
Reçu point 9, recompte aux deux bornes mis à jour. Choix C-15-1a2-19 à 22. Prochaines passes : 15-1a2-0 **P1**,
15-1a2-i **P4**, 15-1a2-ii **P4**.

### Validation P2 — 2026-10-09 (Opus 5.5 ×2 par sous-fiche ; remédiation Opus 5.5, seul remédiateur des trois fiches de la suite du lettrage)

Bilans par finding dans les Change Logs des sous-fiches (et de la 15-1b). 15-1a2-i : 0 HIGH, **4 MEDIUM
distincts** ; 15-1a2-ii : **1 HIGH, 6 MEDIUM distincts** (signal D5 levé, déclaré, non découpé). Décision
structurante : **refus** du geste qui délettrerait un groupe `document` sans ligne en période ouverte —
C-15-1a2-2 révisée deux fois (abstention partout → délettrer toujours, écartée → refus ; C-15-1a2-10).
Dérogation de la 15-1a2-i à la règle de splitting (six modules, C-15-1a2-13). Choix C-15-1a2-10 à 18.
Corps de cet index : table, Reçu point 9 et recompte mis à jour. Prochaine passe : **P3**.

### Validation P1 — 2026-10-09 (Sonnet 5.5 ×2, lentilles R et F ; remédiation Opus 5.5, en autonomie)

Passe 1 : **R** 3 HIGH / 7 MEDIUM / 4 LOW, **F** 0 CRITICAL / 2 HIGH / 7 MEDIUM / 3 LOW (doublons :
R2 = F-6, R3 = F-1, R5 = F-2, R7 = F-4, R8 = F-8, R9 = F-7, R10 = F-13, R1 ≈ F-5) ; **26 findings**, dont
**2 réfutés** (R12 en entier, R10 sur le squash) et le reste corrigé. **Découpage** (F-9, C-15-1a2-1) en
15-1a2-i et 15-1a2-ii ; corps de cette fiche vidé ; numérotation conservée. Choix consignés au registre :
**C-15-1a2-1 à C-15-1a2-9**. Recompte : 12 AC / 7 tâches / 22 tests neufs (i) ; 8 AC / 7 tâches / 14
tests neufs (ii). Prochaine passe : **P2, Opus**, complète, sur chacune des deux sous-fiches.

### Reçu de la 15-1a-i — 2026-10-09 (Opus 5.5, remédiation de la revue de code P2 de la 15-1a-i)

Section « Reçu de la 15-1a-i — revue de code P2 » ajoutée (finding A2-1, registre C-15-1a-i-7 et
C-15-1a-i-11) : six textes provisoires écrits par la 15-1a-i (« Kesh ne lettre rien de lui-même »,
origines « réservées »), dont quatre à réécrire ici au même commit que le groupe `document`. Corps de
cette fiche non réécrit. Édition hors passe. *(Intégré à la 15-1a2-i AC12 par la validation P1.)*

### Reçu de la validation P4 du socle — 2026-10-09 (Opus 5.5, remédiation de la 15-1a-i et de la 15-1a-ii)

Section « Reçu de la 15-1a » complétée des points 18 à 20 (registre C128 à C130) : nom des exercices en
mode `System` lu sans verrou, promesse « à lettrer » retirée de l'écran par la 15-1a-i, réserve « intacte
hors la marque » pour l'origine d'une contre-passation, à transposer aux avoirs. Corps de cette fiche
non réécrit.

### Reçu de la validation P3 du socle — 2026-10-09 (Opus 5.5, remédiation de la 15-1a)

Section « Reçu de la 15-1a » complétée des points 14 à 17 (registre C124 à C127) : la 15-1a est découpée
en 15-1a-i et 15-1a-ii (cette fiche suppose les deux) ; le point 11 est périmé (verrous d'exercice un par
un, bornés au groupe, C125) ; `ENTRY_LETTERED` parle en dernier (C126) ; l'audit porte l'exercice par
ligne (C127). Dépendance et ordre de tête mis à jour ; un numéro du manuel actualisé au point 6. Corps
non réécrit : à intégrer à la validation de cette fiche.

### Création — 2026-10-08 (reprise du lettrage, Opus 5.5, en autonomie)

Story **nouvelle**, née de la relecture du lettrage contre `invoice_settlements` (registre C93,
C98). Elle n'a pas d'ancêtre dans les fiches d'août, qui supposaient qu'aucune écriture
d'encaissement n'existait. **12 critères** (AC1–AC12), **6 tâches** (T1–T6), recomptés depuis ce
fichier.
