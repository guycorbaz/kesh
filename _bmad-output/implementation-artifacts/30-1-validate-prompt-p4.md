# Prompt — validation P4 de la spec, Story 30-1 (la réconciliation règle les factures fournisseur)

*Versionné le 2026-10-15. Passe **complète**, deux lentilles, contexte frais chacune, en lecture seule. Rotation (D6,
rétro 25) : P1 Sonnet → P2 Opus → P3 Sonnet → **P4 Opus**, pour les deux lentilles.*

Dépôt `/home/gcorbaz/devel/kesh-30-1`, branche `story/30-1` (base `15686d55`). Fiche :
`_bmad-output/implementation-artifacts/30-1-reconciliation-factures-fournisseur.md` (HEAD de la branche) — **une fiche,
deux sous-stories** (30-1a puis 30-1b). Registre : `epic-30-choix-autonomes-301.md` (C-30-1-1 à 38). Epic :
`_bmad-output/planning-artifacts/epic-30-factures-fournisseur.md` § *30-1*. Issues, en lecture : `gh api
repos/guycorbaz/kesh/issues/686`, `…/671`, `…/687`, `…/669`, `…/482`. Rapports des passes précédentes (lecture) :
`/home/gcorbaz/devel/kesh-gate-logs/301-validate-p{1,2,3}-{F,R}.md` ; les remédiations sont les entrées « Validation
P1 », « Validation P2 » et « Validation P3 — 2026-10-15 » du Change Log de la fiche.
Règles : `CLAUDE.md` (§ Review Iteration Rule et ses sous-sections — le manuel nommé, les axes déclarés, les sites NON
RÉSOLUS, la propagation post-patch, le recompte des comptes rendus ; § Règle de splitting préventif ; § Pattern batch ;
§ *Un appariement automatique propose, il ne crée jamais* ; § Migration breaking policy) ;
`docs/agents-orchestration.md` ; rétrospective `epic-28-retro-2026-10-14.md` § 4 et § 7 (périmètre minimal ; B5 :
verrous **et** naissance de la vue ; § 4.3 : aucune dette assumée sans issue).

## Axe obligatoire, commun aux deux lentilles — la remédiation P3, relue ligne à ligne contre le code

Prends le diff de la P3 —
`git diff da6d6ecc HEAD -- _bmad-output/implementation-artifacts/30-1-reconciliation-factures-fournisseur.md` — et, **pour chaque ligne ajoutée ou réécrite qui affirme un fait du code** (chemin, numéro de
ligne, ordre de verrous, comportement, nom de fonction, code d'erreur, compteur, contrat de route), ouvre le code et
vérifie-la (`sed -n`, `grep -nF`, sortie copiée). **Tout défaut du code existant que tu invoques se cite par sa ligne.**
Puis cherche le défaut que chaque prescription neuve introduit :
- **Règlement en attente de rapprochement** (AC-b3, troisième fonction ; AC-b5, deuxième ligne ; `notice`
  `supplier_payment_awaiting_reconciliation`) — le critère (virement de ce compte, montant exact, écriture non
  rapprochée, ±30 jours) est-il juste au schéma (`supplier_invoices`, `journal_entries.entry_date`,
  `bank_transactions.matched_entry_id`) ? Sur-blocage (abonnement payé chaque mois depuis la fiche, jamais rapproché :
  chaque débit du mois suivant est-il bloqué par le règlement du mois précédent, à 30 jours ?) ; sous-blocage (frais,
  compte interne) ; le texte « Kesh ne sait pas encore les rapprocher (#671) » est-il vrai, et la ligne a-t-elle encore
  un geste sûr (*Rejeter* : la transaction reste `pending` et bloque la clôture du compte) ?
- **Garde (d) de l'acceptation** (AC-b6) — lectures verrouillantes après le compte : verrous d'intervalle neufs
  (sur quels index ?), cycles avec `pay`, `create_batch`, `confirm_batch`, le dé-rapprochement ; « aucun lot ni
  règlement de ce compte ne peut se commettre entre (d) et la fin du lot » est-il vrai au code (`create_batch` attend-il
  le compte **avant** d'insérer ses articles ? `pay` avant son `UPDATE` ?) ; DRY réel (une fonction, deux lecteurs, deux
  formes de lecture) ?
- **AC-a9 révisé** — liste vide court-circuitée, borne de 500 (contrat de `POST /payment-batches` : un client qui envoie
  davantage aujourd'hui ?), plan épinglé (`WHERE id IN (…) AND company_id = ?` parcourt-il bien `PRIMARY` ?), aide
  `attendre_des_requetes_en_cours`, T-a15 et T-a16 rouges au sol par leur chorégraphie ?
- **AC-b17 (ex-L7)** — `accept_one_rule` rejoue le masquage : les fonctions sont-elles réellement **les mêmes** que
  celles de `get_proposals` (DRY), et la condition de la 29-6 rejouée est-elle identique (`propose_matches` non vide sur
  les factures dépassées de 1 à 5 centimes) ? Lectures et naissance de la vue dans `accept_one_rule` ; ordre des
  verrous inchangé ? Changement de contrat client (une acceptation de règle autrefois réussie, désormais refusée) : dit à
  `api-external.md` et au CHANGELOG ? Le maintien de l'acceptation d'une règle contre une facture **candidate** est-il
  un faux rattachement muet ? T-b46, T-b47 et M-b30 tuables ? La 30-1b reste-t-elle à 5 modules ?
- **Libellés** (AC-b11) — décompte 37, bras `INACTIVE_OR_INVALID_ACCOUNTS`, deux `notice` en quatre locales, clés à
  variante ; `lint-i18n-ownership` ?
- **Manuel** (AC-b13) — FAQ `:3034`, `:3040-3045`, `:2156` : ce que la fiche prescrit sera-t-il vrai ?
- **Recomptes** : 26 AC, 24 tâches, 62 tests nommés (7 Vitest) plus un E2E à trois tests, 39 lignes de mutation (36 à
  exécuter), 38 choix ; modules 5 et 5 « au seuil » avec quatre fichiers hors décompte — recompte chacun.

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
  **qui rougirait** au sol ; chaque fait, chemin et numéro de ligne de la remédiation P3 **revérifié** ; les registres
  (`bank_account_closed_inventory.rs` — l'AC-a9 ajoute-t-il un écrivain ou change-t-il un site ? —,
  `letterings_lexical.rs`, `audit_route_registry.rs`, `textes_coherents.rs`, `reconciliation_e2e.rs`,
  `failed-proposal-label.ts` et son test, `lint-i18n-ownership`) ; la faisabilité des trois E2E (AC-b12).
- **F — Faisabilité et exactitude de la conception (adversaire)** : axe obligatoire et axes 2, 3 en profondeur ; les
  courses T-a6, T-a11, T-a15, T-a16, T-b22, T-b24, T-b25, T-b34, T-b45 et le rejeu T-a14 — chacun rouge au sol, et seulement sous sa
  mutation ? ; les mutations M-a1 à M-b29 (tuabilité, équivalences, M-b27 déclarée non tuable à bon droit ?) ; l'absence de migration (AC-b15).

## Ce que tu rends

Findings numérotés (`R1-…` ou `F1-…`) avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`,
**preuve** (commande exécutée et sortie copiée), correction proposée. Toute affirmation de présence ou d'absence =
sortie de `grep -nF` copiée. ⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne compte
pas comme passe. Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/301-validate-p4-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/301-validate-p4-F.md` (lentille F), **écrit d'un seul jet** par l'outil d'écriture
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
