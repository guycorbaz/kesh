# Prompt — revue de code P2 ciblée, Story 15-5c

*Versionné le 2026-10-08. Passe ciblée (CLAUDE.md § « La passe ciblée ») : une seule lentille (Haiku), contexte
frais, en lecture seule, braquée sur la seule remédiation de la revue P1.*

Worktree `/home/gcorbaz/devel/kesh-15-5c`, branche `story/15-5c-rapprochement-libelles-et-manuel`. **Objet** :
`git show 1f481654` (un seul commit). Rapports remédiés : `target/gate-logs/15-5c-review-p1-{B,E,A}.md`. Choix
C-15-5c-3 au registre `epic-15-choix-autonomes.md`.

## Lentille unique — chasseur de régressions de la remédiation

1. **Libellé `reconciliation-failed-payment-before-invoice`** : la chaîne `payment_date_before_invoice_date` est-elle
   bien la seule posée pour ce cas, et posée telle quelle dans `details.reason` (`grep -rnF
   "payment_date_before_invoice_date" crates/`) ? Le code frontend la lit-il au bon endroit et retombe-t-il sur le
   libellé générique pour toute autre raison, ou un `details` absent ou malformé ? 4 locales, mêmes variables, repli
   Svelte identique au français.
2. **`clearBatchReport()`** : appelée au début de **tout** nouveau bilan (lot, affectation manuelle, éclatement) et
   nulle part où elle effacerait un bilan qu'on doit encore voir ? La doublure `ModalSuccessStub.test.svelte` est-elle
   vraiment exclue du build et des relevés i18n (vérifie les globs) ?
3. **Tests** : le repli `TX #99` et les tests de remise à zéro mordent-ils (une mutation les ferait-elle rougir) ?
4. **Manuel** (`.tex` + PDF aplati : `pdftotext -nopgbrk docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`
   vers `target/gate-logs/`) : le paragraphe « Une facture proposée peut être refusée à l'acceptation » dit-il vrai
   (date de valeur, borne d'un jour, `invoice_settlements_write.rs:96`) ? Classes 5, 6 et 7, « 100 % ». Compteur
   `sitesTotal` 1895 → 1896 cohérent avec les appels `i18nMsg(` ajoutés.

## Ce que tu rends

Findings numérotés (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (sortie de `grep -nF` ; code relu cité),
correction proposée. ⛔ **Liste des axes exercés ET non exercés.** Rapport complet dans
`target/gate-logs/15-5c-review-p2-ciblee.md` ; dernier message : le chemin, le bilan, une ligne par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `target/gate-logs/`. Aucune commande qui écrit dans le dépôt ou dans une base, ni qui
compile ou exécute des tests : `scripts/*` (dont `scripts/prepare-release.sh`), `make`, `latexmk`,
`git commit`/`add`/`checkout`/`stash`, `sqlx`, `cargo`, `npm`, `npx`, `gh issue create`/`comment`/`edit`, SQL
d'écriture. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`, `gh api` en lecture, `pdftotext` vers
`target/gate-logs/`.
