# Prompt — validation P5 de la spec, Story 30-1 (la réconciliation règle les factures fournisseur)

*Versionné le 2026-10-15. Passe **complète**, deux lentilles, contexte frais chacune, en lecture seule. Rotation (D6,
rétro 25) : P1 Sonnet → P2 Opus → P3 Sonnet → P4 Opus → **P5 Sonnet**, pour les deux lentilles. Passe **complète** et non ciblée :
la remédiation P4 change une règle métier (une garde retirée, une garde ajoutée à l'acceptation des règles).*

Dépôt `/home/gcorbaz/devel/kesh-30-1`, branche `story/30-1` (base `15686d55`). Fiche :
`_bmad-output/implementation-artifacts/30-1-reconciliation-factures-fournisseur.md` (HEAD de la branche) — **une fiche,
deux sous-stories** (30-1a puis 30-1b). Registre : `epic-30-choix-autonomes-301.md` (C-30-1-1 à 39 ; C-30-1-34 **retiré**). Epic :
`_bmad-output/planning-artifacts/epic-30-factures-fournisseur.md` § *30-1*. Issues, en lecture : `gh api
repos/guycorbaz/kesh/issues/686`, `…/671`, `…/687`, `…/669`, `…/482`. Rapports des passes précédentes (lecture) :
`/home/gcorbaz/devel/kesh-gate-logs/301-validate-p{1,2,3,4}-{F,R}.md` ; les remédiations sont les entrées « Validation
P1 » à « Validation P4 — 2026-10-15 — et simplification » du Change Log de la fiche.
Règles : `CLAUDE.md` (§ Review Iteration Rule et ses sous-sections — le manuel nommé, les axes déclarés, les sites NON
RÉSOLUS, la propagation post-patch, le recompte des comptes rendus ; § Règle de splitting préventif ; § Pattern batch ;
§ *Un appariement automatique propose, il ne crée jamais* ; § Migration breaking policy) ;
`docs/agents-orchestration.md` ; rétrospective `epic-28-retro-2026-10-14.md` § 4 et § 7 (périmètre minimal ; B5 :
verrous **et** naissance de la vue ; § 4.3 : aucune dette assumée sans issue).

## Axe obligatoire, commun aux deux lentilles — la simplification, relue contre le code, et l'absence de résidu

Signal de recyclage (P4) : la classe « double comptage » revenait à chaque passe sous une forme née de nos propres
gardes. L'orchestrateur a **simplifié** (C-30-1-39). Prends le diff —
`git diff 3e900779 HEAD -- _bmad-output/implementation-artifacts/30-1-reconciliation-factures-fournisseur.md` — et :
1. **Pour chaque ligne ajoutée ou réécrite qui affirme un fait du code**, ouvre le code et vérifie-la (`sed -n`,
   `grep -nF`, sortie copiée). **Tout défaut du code existant que tu invoques se cite par sa ligne.**
2. **Absence de résidu** de ce qui est retiré : `grep -nE` sur la fiche **hors des entrées datées du Change Log** et des
   lignes barrées — `supplier_payment_hold`, `awaiting`, `±30`, `HeldByAwaiting`, `supplier_payment_awaiting`,
   « verrouillante » appliqué à (d) ou à `accept_one_rule`, `T-b43`, `T-b44`, `M-b28`, « deux `notice` » ; et sur le
   registre (C-30-1-34 doit être dit retiré, C-30-1-35 annoté). Un résidu qui prescrit encore ce qui est retiré est au
   moins **MEDIUM**.
3. **La simplification ne rouvre-t-elle pas un défaut fermé ?**
   - **L1 redevenu angle mort** : le texte du manuel prescrit (AC-b13) est-il **juste** au code — « annulez le paiement
     depuis la fiche de la facture : elle vous sera alors proposée ici » (l'annulation est-elle permise pour un paiement
     non rapproché, et par un lot confirmé ? la facture redevenue ouverte est-elle candidate si le débit est dans la
     fenêtre ?) ; L1 est-il **préexistant** à la lettre (v0.13.4) ou la 30-1 l'aggrave-t-elle (la proposition masque-t-
     elle désormais moins qu'avant) ?
   - **Lectures ordinaires** de (d) et d'`accept_one_rule` : la tolérance écrite (« un lot commis après la vue n'est pas
     vu ») suffit-elle, ou laisse-t-elle un double paiement réel (T-b22 / M-b8 couvrent (c), la facture acceptée) ?
     L'ordre des verrous de la table de l'AC-b10 est-il exact, sans cycle neuf ?
   - **Une seule fonction du lot** : trois lecteurs, un seul texte ; `excluding` à l'acceptation (la facture acceptée,
     couverte par (c)) — juste ?
   - **AC-b17** : la reprise de `has_strong_invoice_match` à l'acceptation d'une règle est-elle **la même** que la
     proposition (même requête des candidates, même fenêtre, même seuil) ? Le changement de contrat (règle refusée)
     est-il dit partout (manuel `:2156`, `api-external.md`, CHANGELOG *Modifié*, libellés) ? Une intégration qui
     acceptait des règles par l'API perd-elle un geste sans alternative (*Affecter manuellement* le remplace-t-il au
     code, `POST /reconciliation/manual`) ?
   - **Masquage restant** (AC-b5) : critère des lignes 2 à 5 cohérent avec la requête masquante (sans indicateur de lot)
     et avec la candidate retirée par identifiant ?
4. **Recomptes** : 26 AC (9 + 17), 24 tâches, 61 tests nommés (7 Vitest) plus un E2E à trois tests, 39 lignes de
   mutation (36 à exécuter), 39 choix, 37 codes de libellé, modules 5 et 5 — recompte chacun depuis la fiche.

Un défaut introduit par la simplification est au moins **MEDIUM** ; un faux rattachement ou un double comptage muet que
la **30-1** crée (et non un préexistant écrit en angle mort avec son issue), ou un cycle non rejoué, est **HIGH**.

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
  **qui rougirait** au sol ; chaque fait, chemin et numéro de ligne de la simplification P4 **revérifié** ; les registres
  (`bank_account_closed_inventory.rs` — l'AC-a9 ajoute-t-il un écrivain ou change-t-il un site ? —,
  `letterings_lexical.rs`, `audit_route_registry.rs`, `textes_coherents.rs`, `reconciliation_e2e.rs`,
  `failed-proposal-label.ts` et son test, `lint-i18n-ownership`) ; la faisabilité des trois E2E (AC-b12).
- **F — Faisabilité et exactitude de la conception (adversaire)** : axe obligatoire et axes 2, 3 en profondeur ; les
  courses T-a6, T-a11, T-a15, T-a16, T-b22, T-b24, T-b25, T-b34, T-b45, T-b46, T-b47 et le rejeu T-a14 — chacun rouge au sol, et seulement sous sa
  mutation ? ; les mutations M-a1 à M-b30 (tuabilité, équivalences ; M-b27 réécrite — tuée par T-b34 à bon droit ? ; M-b28 retirée) ; l'absence de migration (AC-b15).

## Ce que tu rends

Findings numérotés (`R1-…` ou `F1-…`) avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`,
**preuve** (commande exécutée et sortie copiée), correction proposée. Toute affirmation de présence ou d'absence =
sortie de `grep -nF` copiée. ⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne compte
pas comme passe. Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/301-validate-p5-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/301-validate-p5-F.md` (lentille F), **écrit d'un seul jet** par l'outil d'écriture
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
