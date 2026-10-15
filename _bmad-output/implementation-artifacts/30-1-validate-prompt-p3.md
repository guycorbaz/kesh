# Prompt — validation P3 de la spec, Story 30-1 (la réconciliation règle les factures fournisseur)

*Versionné le 2026-10-15. Passe **complète**, deux lentilles, contexte frais chacune, en lecture seule. Rotation (D6,
rétro 25) : P1 Sonnet → P2 Opus → **P3 Sonnet**, pour les deux lentilles.*

Dépôt `/home/gcorbaz/devel/kesh-30-1`, branche `story/30-1` (base `15686d55`). Fiche :
`_bmad-output/implementation-artifacts/30-1-reconciliation-factures-fournisseur.md` (HEAD de la branche) — **une fiche,
deux sous-stories** (30-1a puis 30-1b). Registre : `epic-30-choix-autonomes-301.md` (C-30-1-1 à 33 ; amendés : 11, 12,
14, 16, 20, 27). Epic : `_bmad-output/planning-artifacts/epic-30-factures-fournisseur.md` § *30-1*. Issues, en
lecture : `gh api repos/guycorbaz/kesh/issues/686`, `…/671`, `…/687`, `…/482`. Rapports des passes précédentes
(lecture) : `/home/gcorbaz/devel/kesh-gate-logs/301-validate-p1-{F,R}.md`, `…-p2-{F,R}.md` ; les remédiations sont les
entrées « Validation P1 » et « Validation P2 — 2026-10-15 » du Change Log de la fiche.
Règles : `CLAUDE.md` (§ Review Iteration Rule et ses sous-sections — le manuel nommé, les axes déclarés, les sites NON
RÉSOLUS, la propagation post-patch, le recompte des comptes rendus ; § Règle de splitting préventif ; § Pattern batch ;
§ *Un appariement automatique propose, il ne crée jamais* ; § Migration breaking policy) ;
`docs/agents-orchestration.md` ; rétrospective `epic-28-retro-2026-10-14.md` § 4 et § 7 (périmètre minimal ; B5 :
verrous **et** naissance de la vue ; § 4.3 : aucune dette assumée sans issue).

## Axe obligatoire, commun aux deux lentilles — la remédiation P2, relue ligne à ligne contre le code

Les passes P1 et P2 ont surtout trouvé des défauts **écrits par la remédiation précédente**. Prends le diff de la P2 —
`git diff f53fa0f2 HEAD -- _bmad-output/implementation-artifacts/30-1-reconciliation-factures-fournisseur.md` — et,
**pour chaque ligne ajoutée ou réécrite qui affirme un fait du code** (un chemin, un numéro de ligne, un ordre de
verrous, un comportement, un nom de fonction, un code d'erreur, un compteur), ouvre le code et vérifie-la (`sed -n`,
`grep -nF`, sortie copiée). Puis cherche le défaut que chaque prescription neuve introduit :
- **AC-b5, première ligne** (lot `generated` du compte : aucune proposition) — sur-blocage ? (un débit **réellement**
  destiné à une autre facture, ou à une règle, privé de proposition : la `notice` suffit-elle à le dire et au remède ?) ;
  la ligne est-elle bornée au **compte** et au **montant exact** au code prescrit ? ; que voit l'utilisateur après la
  confirmation du lot (la facture devient `paid`, la ligne redevient-elle proposable — et à quoi ? cf. L1) ?
- **AC-b3, requête masquante** — tri, plafond 200, « liste pleine = masquante », exécution sans règle active : coût
  (une requête de plus par débit, sur 100 à 500 transactions) et index ; le partage du texte SQL avec les candidates
  tient-il avec un `ORDER BY` différent ?
- **AC-a9, `create_batch`** — le verrouillage en tête est-il réellement mécanique (lire `payment_batches.rs:69-200`) ?
  Les codes et l'ordre de `failed[]` sont-ils préservés ? Le verrou d'intervalle d'une facture absente, la clé primaire,
  peut-il fermer un cycle avec `pay`, `confirm_batch`, l'acceptation ? T-a15 est-il rouge au sol par la chorégraphie
  écrite ?
- **AC-a8, rejeu** — `audit_route_registry.rs` (vi) et (iii bis) : le décompte « quinze » est-il juste au code ?
  `api-external.md:788` : la phrase prescrite est-elle vraie ?
- **AC-b6, étape 6** — l'argument « le règlement ne valide aucun projet » (lignes `:875`, `:881`, `journal_entries.rs:
  300-303`) et la conséquence `INTERNAL_ERROR` ; les conventions de `details` (signes, casse) contre les refus client
  existants.
- **AC-b10, T-b34** — la chorégraphie est-elle exécutable avec les sondes du dépôt, et rouge sous M-b23 seulement ?
  La table complétée (clôture, `settle_invoice`, annulations fournisseur, `POST /bank-imports`, C7) est-elle exacte ?
- **AC-a6, texte commun** — l'énoncé « bras serveur inatteignable » (`errors.rs:3703`) est-il vrai ? La réécriture de
  `errors.rs:4237-4238` ?
- **AC-b1, `payment_reference` libre** — la prémisse corrigée, la borne de 6, la garantie restreinte à la référence.
- **Découpage** — le déplacement du dialogue (C-30-1-33) : la 30-1a reste-t-elle cohérente (vue et `…/cancel` livrés
  sans l'écran qui les lit) ? Modules : 5 et 5 au code prescrit ?
- **Recomptes** : 25 AC (9 + 16), 24 tâches, 56 tests nommés (7 Vitest) plus un E2E à trois tests, 34 lignes de mutation
  (32 à exécuter), 33 choix — recompte chacun depuis la fiche.

Un défaut introduit par la remédiation est au moins **MEDIUM** ; un faux rattachement ou un double comptage muet, ou un
cycle non rejoué, est **HIGH**.

## Axes prioritaires (repris, sur l'état actuel)

1. **Prémisses** (§ *Ce qui existe*, § *Prémisses de #686*) : revérifie au code.
2. **Aucun faux rattachement muet** (règle du `CLAUDE.md`, #482, leçon de la 29-6) : proposition, acceptation directe
   par l'API (le refus du lot `generated` à l'étape 3 couvre-t-il ce que la proposition efface ?), lot mixte.
3. **Verrous et naissance de la vue (B5)** : AC-a4, AC-a9, AC-b8, AC-b10.
4. **DRY** : un jumeau prescrit, ou une généralisation qui change un comportement client, est au moins **MEDIUM**.
5. **Manuel contre code** (obligatoire) : `docs/manual/fr/user-manual.tex` l. 1586-1640, 2111-2160, 2252 et suivantes,
   et le **PDF**, aplati vers la **sortie standard** seulement : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n'
   ' ' | tr -s ' '` (⛔ jamais vers un fichier) ; ce que l'AC-b13 prescrit (la `notice` et son remède, le lot confirmé
   de L1, `paidAt`) sera-t-il vrai ? Même exigence pour `docs/api-external.md` (`:366`, `:572-612`, `:788`), le
   CHANGELOG `## [0.14.0] — Non publié` et le README.

## Lentilles

- **R — Relecture contre le code au sol (auditeur d'acceptation)** : `.claude/skills/bmad-create-story/checklist.md` ;
  chaque phrase de #686 et du § *30-1* de l'epic couverte par un AC testable, chaque AC par une tâche et par un test
  **qui rougirait** au sol ; chaque fait, chemin et numéro de ligne de la remédiation P2 **revérifié** ; les registres
  (`bank_account_closed_inventory.rs` — l'AC-a9 ajoute-t-il un écrivain ou change-t-il un site ? —,
  `letterings_lexical.rs`, `audit_route_registry.rs`, `textes_coherents.rs`, `reconciliation_e2e.rs`,
  `failed-proposal-label.ts` et son test, `lint-i18n-ownership`) ; la faisabilité des trois E2E (AC-b12).
- **F — Faisabilité et exactitude de la conception (adversaire)** : axe obligatoire et axes 2, 3 en profondeur ; les
  courses T-a6, T-a11, T-a15, T-b22, T-b24, T-b25, T-b34 et le rejeu T-a14 — chacun rouge au sol, et seulement sous sa
  mutation ? ; les mutations M-a1 à M-b26 (tuabilité, équivalences) ; l'absence de migration (AC-b15).

## Ce que tu rends

Findings numérotés (`R1-…` ou `F1-…`) avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`,
**preuve** (commande exécutée et sortie copiée), correction proposée. Toute affirmation de présence ou d'absence =
sortie de `grep -nF` copiée. ⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne compte
pas comme passe. Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/301-validate-p3-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/301-validate-p3-F.md` (lentille F), **écrit d'un seul jet** par l'outil d'écriture
de fichier : ⛔ aucun `sed -i`, `perl -pi`, ni retouche par commande, **même sur ton propre rapport** — s'il faut le
corriger, réécris-le en entier. Dernier message : le chemin du rapport, le bilan par sévérité, une ligne par finding
MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/` — ni sortie de `pdftotext`, ni
note, ni copie. Aucune commande qui écrit, compile ou exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**,
`scripts/test-fast.sh`, `scripts/mem-guard.sh` et les scripts Synology), `make`, `latexmk`, `xelatex`, `sed -i`,
`perl -pi`, `git commit`/`add`/`checkout` **sous aucune forme** (`--detach` compris)/`switch`/`restore`/`stash`/
`fetch`/`pull`/`reset`/`rebase`/`tag`/`push`/`worktree`, `cargo`, `npm`, `npx`, `node`, **tout interpréteur**
(`python3`, `python`, `perl`, `ruby`, `bash`/`sh` sur un script, `awk` qui écrit), `docker`, **`mariadb`/`mysql`** et
toute requête SQL, `gh` en écriture. Aucune donnée réelle de Guy ne se lit ni ne se cite. Autorisés : lecture, `grep`,
`sed -n`, `git log`/`show`/`diff`/`blame`, `gh api` en lecture, `pdftotext` vers la sortie standard.
