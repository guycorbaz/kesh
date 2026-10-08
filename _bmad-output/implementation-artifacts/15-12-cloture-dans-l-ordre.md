# Story 15.12 : Clôturer les exercices dans l'ordre — et ne plus écrire sous un bilan clos — fiche index, découpée en 15-12a et 15-12b

Status: split

<!-- Découpée le 2026-10-09 à la remédiation de la validation P2, décision de l'orchestrateur — choix
     C107 (`epic-15-choix-autonomes.md`) : la dérogation au découpage (C100) ne relevait pas de
     l'exception que le `CLAUDE.md` codifie (F3), et la dépendance entre les deux moitiés n'était pas à
     sens unique (R3). Corps de la fiche vidé : la **version complète avant découpage** est au commit
     **dae3a618** (`git show dae3a618:_bmad-output/implementation-artifacts/15-12-cloture-dans-l-ordre.md`).
     Le Change Log de la création et de la validation P1 est conservé ci-dessous ; la remédiation P2 est
     appliquée dans les deux fiches filles. -->

**Issue** : #543 (P1) — **fermée par la 15-12b** ; la 15-12a porte `refs #543`.

## Découpage

| fiche | contenu | issue | dépend de |
|---|---|---|---|
| **15-12a** — `15-12a-cloture-dans-l-ordre.md` | **L'ordre** : l'invariant I (les exercices clos forment un préfixe) et ses trois transitions — clôture refusée tant qu'un antérieur est ouvert (`409 EARLIER_FISCAL_YEAR_OPEN`), création refusée sous un postérieur clos (`400 LATER_FISCAL_YEAR_CLOSED`), réouverture LIFO inchangée ; ordre des verrous de `close` tenu par le code — antérieurs verrouillés un par un par clé primaire, puis relecture verrouillante (C119, qui révise C110) ; rejeu des deux routes et preuve HTTP de l'enveloppe (C109) ; **message neutre** de `LATER_FISCAL_YEAR_CLOSED` et ancienne clé alignée (AC 9, C111) ; tests de concurrence à deux connexions ; bouton « Clôturer » désactivé ; E2E ; doc de la clôture. AC 1-7, 9, 13, 14, 17, 22 et la part A des AC 19, 21, 23 — 15 AC, 11 tâches | refs #543 | 15-8a, 15-8b, 15-5e1/e2 (mergées) |
| **15-12b** — `15-12b-filet-sous-un-bilan-clos.md` | **Le filet** pour les données héritées : `LATER_FISCAL_YEAR_CLOSED` aux deux points de passage du journal (`create_in_tx_inner` sans verrou, `delete_in_tx` sans condition — la dévalidation), inventaire fermé des écrivains, lot de rapprochement aux **trois** voies (C108), bandeau d'état hérité et réparation dans l'ordre LIFO, autres écrans et prédicteurs (angle mort, C120), doc du filet. AC 8, 10, 11, 12, 15, 16, 18, 20 et la part B des AC 19, 21, 23 — 11 AC, 9 tâches | **closes #543** | **15-12a** |

**Numérotation** : les AC gardent leur numéro de la 15-12 dans les deux fiches (références croisées
stables) ; un AC partagé porte « part A » / « part B ». Les tâches sont renumérotées par fiche, chacune
avec son origine (« ex-T4 »).

**Recompte aux deux bornes** (commandes : `sed -n '/^## Acceptance/,/^## Tasks/p' <fiche> | grep -oE
'^ ?[0-9]{1,2}\. \*\*'` et `grep -cE '^- \[ \] \*\*T[0-9]+' <fiche>`) — avant (`dae3a618`) : **23 AC,
14 tâches** ; après : 15-12a **15 AC, 11 tâches**, 15-12b **11 AC, 9 tâches**. 15 + 11 = 26 = 23 + 3 AC
partagés (19, 21, 23) ; 11 + 9 = 20 = 14 + 6 tâches présentes dans les deux fiches (ex-T0, T7, T9, T11,
T12, T13), l'ex-T4 étant répartie (le message dans la T3 de la 15-12a, le filet dans la T1 de la 15-12b).
Mutations : (i)-(iv), (viii), (ix) et la neuve (x) dans la 15-12a ; (v), (vi), (vii-a), (vii-b) dans la
15-12b.

**Ordre écrit** (C112, mis à jour au découpage de la 15-1a, C125) : **15-12a → 15-12b → 15-1a-i → 15-1a-ii** (puis 15-1b, 15-1c). Le **prérequis réel** de la
15-1a est la **15-12a** (l'invariant I, la clôture rejouée) ; la 15-12b passe avant de préférence, les
deux touchant `journal_entries::delete_in_tx`. ⚠️ La v0.13.0 ne se tague pas sans la 15-12b (#543, P1 ;
et le message neutre de la 15-12a n'est vrai de l'état hérité qu'avec le filet).

## Change Log

- 2026-10-08 — Création (bmad-create-story, en autonomie ; choix C89). 23 AC, 14 tâches (T0-T13).
  Inventaires : 22 routes `Rejouee` + 2 `Exemptee` de production au registre, ramenées à 3 points de
  passage (19 / 1 / 2, plus 2 hors filet) ; sources de l'état fautif : clôture hors ordre, création
  d'un exercice antérieur à un exercice clos, restauration d'une sauvegarde (réouverture : garde LIFO ;
  démo, graine de test et `onboarding::finalize` : un seul exercice, sans risque). Dérogation au
  découpage écrite (8 modules), coupe de repli 15-12a/15-12b.
- 2026-10-09 — **Validation P1** (deux lentilles Opus en parallèle, contexte frais : R auditeur
  d'acceptation, F adversaire plein périmètre ; rapports `target/gate-logs/15-12-p1-R.md` et
  `…-p1-F.md`). Bruts : R 1 HIGH / 3 MEDIUM / 11 LOW, F 0 / 5 MEDIUM / 7 LOW ; après dédoublonnage
  (R1=F1, R2=F2, R3=F3, R4=F6 ; parmi les LOW R8=F9, R11≈F11, R13≈F10, R15=F8) : **1 HIGH, 4 MEDIUM,
  14 LOW distincts**, tous d'origine, **aucun recyclé**. Remédiation (C100) : **HIGH R1=F1** — la
  lecture verrouillante de la clôture ne compte que dans la course réouverture/clôture : AC 13 réécrit
  (13 a ordonné, réouverture de N en cours contre `close(L)`, tue la mutation (ii) sur l'état final ;
  ancien 13 c retiré), preuve des Dev Notes refondue en (α)/(β), AC 8 et AC 19 alignés ; **R2=F2** —
  13 c (ex-13 d) rendu déterministe par un ordre forcé ; **R3=F3** — bandeau : clôturer d'abord
  l'exercice ouvert le plus ancien, sinon rouvrir à partir du plus récent clos, jamais `{closed}` en
  premier ; message de l'AC 9 neutre (F12), réparation directe ajoutée à l'AC 16 ; **F4** — « toutes
  les autres acquisitions sont ascendantes » retiré (AC 3, AC 14) ; ordre réel de la contre-passation
  et des quatre annulations écrit, cycle avec la clôture résolu par le rejeu, test 13 d et mutation
  (ix) ; **R4=F6** — AC 23 : inventaire par recherche des tableaux et listes de refus
  d'`api-external.md` (dévalidation et son décompte `:311`, `:321-391`, `:355`, `:363`, `:371`,
  `:484`) et du manuel (liste de la dévalidation `:1314-1337`) ; **LOW** : précédence de la
  dévalidation (R5), commande de recompte bornée et vérifiée (R6 : 22 / 2), sites dynamiques et
  `delete_all_by_company` tranché (R7), doc-comment du registre (R8/F9), ancienne clé gardée (R9), clé
  du lot tranchée (R10), AC 18 par les appelants avec la boîte de validation d'une facture (R11/F11),
  contre-passation par la route (R12), suspects de `src/` et base partagée (R13/F10), test HTTP de la
  création (R14), dérogation recomptée — 11 modules — et réargumentée sans la preuve réfutée,
  déclencheur de repli élargi (R15/F8), 13 b mesuré dans les deux configurations (F5), restauration en
  vol écrite comme angle mort (F7). Story **non découpée** (C100 : la coupe ne ramène aucune moitié
  sous le seuil — 7 et 8 modules) ; signal déclaré au Project Lead. Toujours 23 AC, 14 tâches
  (recomptés). Passe suivante : P2 complète (Sonnet, rotation D6).
- 2026-10-09 — **Validation P2** (Sonnet ×2, contexte frais : R chasseur de régressions, F adversaire
  plein périmètre ; rapports `target/gate-logs/15-12-p2-R.md` et `…-p2-F.md`). Bruts : R 0 / 0 / 2
  MEDIUM / 5 LOW, F 0 / 0 / 3 MEDIUM / 8 LOW ; après dédoublonnage (R1 = F1, R5 = F6, R7 = F9, R3
  rattaché à F3) : **0 CRITICAL, 0 HIGH, 4 MEDIUM, 10 LOW distincts**. Les MEDIUM sont d'origine ; deux
  LOW sont des résidus de la remédiation P1 (R4, F7). Trend : P1 1 HIGH / 4 MEDIUM → P2 0 HIGH / 4
  MEDIUM. **Story DÉCOUPÉE** (C107, décision de l'orchestrateur sur F3 + R3) en 15-12a / 15-12b, la
  remédiation de chaque finding portée dans la fiche qui convient (C108 à C112 ; détail aux Change Logs
  des deux fiches). F7 (« 8 modules » en tête de la ligne du registre) : ligne réécrite au découpage.
  Point rendu à l'orchestrateur pour la 15-1a (fiche non modifiée) : R6/F11, C112.
  **Passe suivante : P3 complète (Opus ×2, rotation D6) sur les DEUX fiches** — la découpe redistribue
  tout le texte (une omission ou une dépendance A → B résiduelle ne se voit qu'en relisant les deux), et
  la remédiation change des règles (lot à trois voies, `ORDER BY` d'une requête de production, texte du
  refus, preuve de l'enveloppe) : la passe ciblée ne suffit pas (§ *La passe ciblée*).
- 2026-10-09 — **Validation P3** (Opus ×2, contexte frais, sur les **deux** fiches ; rapports
  `target/gate-logs/15-12-p3-R.md` et `…-p3-F.md`). Bruts : R 0 / 0 / 1 MEDIUM / 8 LOW, F 0 / 0 / 3
  MEDIUM / 8 LOW ; après dédoublonnage (R1 = F3, R3 rattaché à F2) : **0 CRITICAL, 0 HIGH, 3 MEDIUM, 15
  LOW distincts**. Le découpage ne perd aucun AC, aucune tâche ni aucune mutation, et aucune dépendance
  A → B résiduelle n'a été trouvée (R). ⚠️ **Deux des trois MEDIUM sont nés de la remédiation P2** (F2 :
  le « par construction » de C110 ; R1 = F3 : l'inventaire par une forme ajouté en P2) — motif du
  `CLAUDE.md`, « la sévérité se déplace vers ce qu'on vient d'écrire ». Trend : P1 1 HIGH / 4 MEDIUM → P2
  0 HIGH / 4 MEDIUM → P3 0 HIGH / 3 MEDIUM. Remédiation (C119 à C123) : **F2** — clôture à verrous un par
  un par clé primaire et relecture verrouillante, `FORCE INDEX` écarté, `OPEN_COVERING_DATE_SQL` sans
  `ORDER BY` (C119, révise C110 ; **à reporter sur C114** de la 15-1a) ; **R1 = F3** — inventaire de
  l'AC 21 part B par le symptôme, un seul site change de sens ; **F1** — prédicteurs d'annulation : angle
  mort assumé, issue (C120) ; **F9** — dérogation au découpage écrite dans la forme codifiée, acceptée par
  l'orchestrateur (C121) ; **C117** reçu de la 15-1a, écrit à la frontière de la 15-12b ; LOW traités
  (détail aux Change Logs des deux fiches). Signal de découpage : la sévérité baisse (4 → 3 MEDIUM), mais
  deux MEDIUM naissent d'un correctif — le **recyclage** que l'amendement D5 nomme comme déclencheur.
  **Déclaré au Project Lead** ; non suivi, la dérogation étant acceptée (C121) — une troisième coupe
  ferait des stories intestables seules ; les deux défauts recyclés portent sur un inventaire et une
  formule, non sur une règle métier qui ne converge pas. Comptes : 15-12a **15 AC / 11
  tâches**, 15-12b **11 AC / 9 tâches**, recomptés par la commande ci-dessus.
  **Passe suivante : P4** — **complète sur la 15-12a** (Sonnet ×2, rotation D6) : C119 change le
  protocole de verrouillage de la clôture, cœur de la fiche, et § *La passe ciblée* exclut la passe
  ciblée quand la remédiation change une règle ; **ciblée sur la 15-12b** (une lentille, Haiku admis par
  D6), braquée sur le seul diff de cette remédiation (AC 11, 12, 18, 21, Dev Notes, dérogation), qui ne
  change aucune règle du filet.
