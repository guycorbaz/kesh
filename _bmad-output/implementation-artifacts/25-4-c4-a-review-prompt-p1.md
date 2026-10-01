# Prompt — revue de code P1, Story 25-4-c4-a (l'arrondi à 5 centimes, figé à la validation)

*Versionné le 2026-10-01. Trois lentilles (Sonnet), contexte frais chacune.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-4-c4-arrondi-facture-5-centimes`. **Diff à revoir :
`git diff 52d367be 04c884db`** (un seul commit d'implémentation, aplati). Fiche :
`_bmad-output/implementation-artifacts/25-4-c4-a-arrondi-fige-a-la-validation.md` (AC 1–8, Dev Notes, Change
Log). CR : `gh issue view 494`. Règles : `CLAUDE.md` (§ *Migration breaking policy* P1–P8, § *Pattern batch*).

## Lentilles

- **A — Blind hunter** : le diff seul. Arithmétique décimale (`round(x × 20) / 20`, `rescale`), signe de
  l'écart dans l'écriture de vente et dans l'avoir, équilibre, lignes à zéro, ordre des lignes, erreurs
  avalées, SQL (alias, jointures, sommes : l'arrondi d'un avoir compté une fois par ligne ? `INNER JOIN` qui
  exclurait un avoir sans ligne ?).
- **B — Edge-case hunter** : diff + code environnant. Réglage changé entre deux validations, dévalidation
  puis revalidation, facture antérieure (arrondi 0) créditée, avoir quand le compte d'arrondi a été archivé,
  facture à plusieurs taux, TTC brut à quatre décimales (10.0050 → 10.00), total arrondi nul, concurrence
  (le `FOR UPDATE` du compte d'arrondi à la validation — ordre des verrous avec la facture, l'exercice, le
  compteur de numéros), sauvegarde antérieure / postérieure, rejeu `post_restore`, backfill
  `20260729000001` (condition (3)), rapprochement et règlement sur une facture arrondie, chemin d'écart au
  centime de la c3-b encore atteignable ?
- **C — Acceptance auditor** : diff + fiche. Chaque AC tenu ? **Pars du symptôme** :
  `grep -rn "invoice_total_ttc\|INVOICE_TTC\|line_ttc_sql!\|INVOICE_CREDITED\|lt\.ttc\|total_ttc" crates/ frontend/src` —
  reste-t-il une lecture du TTC d'une facture ou d'un avoir **émis** qui ignore l'arrondi (rapports,
  `kesh-report`, exports, PDF d'avoir, e-mail de rappel, échéancier) ? Les tests ajustés : chacun dit-il
  pourquoi, et le réglage n'a-t-il été désactivé QUE pour des factures émises sans arrondi par construction ?
  Garde-fous P5/P6/P7/P8 (audit recompté, `migrations_upgrade_path.rs`, `migrations.sha384`, squash).
  CHANGELOG : dit-il vrai ? Le manuel n'est pas touché ici (c4-b) — mais **dit-il quelque chose de faux
  maintenant** sur le montant de la QR ou le total ? (`docs/manual/fr/*.tex`, PDF aplati :
  `pdftotext … | tr '\n' ' ' | tr -s ' '` vers
  `/tmp/claude-1000/-home-gcorbaz-devel-kesh/379e6f94-8029-42cb-9720-fa27c2fb204c/scratchpad/`).

## Ce que tu rends

- **Findings** : sévérité (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (commande exécutée et sa
  sortie, ou code cité relu), scénario d'échec concret, correction proposée. Pour tout finding affirmant
  qu'un code est absent ou présent : la sortie d'un `grep -nF`.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.**

## Interdits

⛔ N'écris aucun fichier du dépôt ; aucune commande qui écrit dans le dépôt ou dans une base : `scripts/*`
(dont `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/regen-test-schema.sh`), `make`,
`latexmk`, `git commit`/`add`/`stash`/`reset`/`checkout`/`switch`/`rebase`, `sqlx migrate`,
`cargo test`/`nextest`, `npm run`, `npx`, `gh issue create`/`comment`/`edit`. Autorisés : lecture, `grep`,
`git log`/`show`/`diff`, `gh issue view`, `cargo check`, `pdftotext` et expériences dans le scratchpad.
