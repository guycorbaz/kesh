# Prompt — revue de code P2 ciblée, Story 15-5d

*Versionné le 2026-10-08. Passe ciblée (CLAUDE.md § « La passe ciblée ») : une seule lentille (Haiku), contexte
frais, en lecture seule, braquée sur la seule remédiation de la revue P1.*

Worktree `/home/gcorbaz/devel/kesh-15-5d`, branche `story/15-5d-garde-usage-comptes-reglage`. **Objet** :
`git show a8bab77b` (un seul commit). Rapports remédiés : `target/gate-logs/15-5d-review-p1-{B,E,A}.md`. Choix
C-15-5d-5, C-15-5d-6.

## Lentille unique — chasseur de régressions de la remédiation

1. **`FORCE INDEX (PRIMARY)`** sur la requête qui verrouille les comptes désignés : syntaxe MariaDB juste
   (`FROM accounts FORCE INDEX (PRIMARY) WHERE … ORDER BY id LOCK IN SHARE MODE`) ? toujours `LOCK IN SHARE MODE`
   et jamais `FOR UPDATE` ? Les motifs `ACCESSEUR` des tests de place 1 et 2 suivent-ils exactement le nouveau texte
   (une divergence ferait attendre puis paniquer, ou au contraire capter une autre requête) ?
2. **Tests neufs** `archived_account_wins_over_a_non_postable_one` et `mode_purchase_lock_is_shared` : mordent-ils
   (mutation annoncée) ? la bloqueuse lit-elle `name` (piège de l'index couvrant `fk_accounts_parent` évité) ?
3. **Doc-comments** de l'accesseur (angle mort E4), de `invoices.rs` et `supplier_invoices.rs` (B-3 : plus de
   « disjoints par type » ; cycle possible à l'achat écrit comme angle mort couvert par le rejeu) ; types
   `InvoiceSettingsResponse` / `UpdateInvoiceSettingsRequest`.
4. **Manuels** : paragraphe de validation du manuel utilisateur (l'avoir n'est pas contrôlé, renvoi « Rôles des
   comptes ») et phrase « Décompte TVA » remise en place au manuel administrateur — `.tex` et PDF aplati
   (`pdftotext -nopgbrk f.pdf - | tr '\n' ' ' | tr -s ' '`, ligatures normalisées) vers `target/gate-logs/`.

## Ce que tu rends

Findings numérotés (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (sortie de `grep -nF` ; code relu cité),
correction proposée. ⛔ **Liste des axes exercés ET non exercés** (ce que tu n'as pas exécuté compte comme non
exercé) — un « 0 » sans elle ne compte pas. Rapport complet dans `target/gate-logs/15-5d-review-p2-ciblee.md` ;
dernier message : le chemin, le bilan, une ligne par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `target/gate-logs/`. Aucune commande qui écrit dans le dépôt ou dans une base, ni qui
compile ou exécute des tests : `scripts/*` (dont `scripts/prepare-release.sh`), `make`, `latexmk`,
`git commit`/`add`/`checkout`/`switch`/`stash`, `sqlx`, `cargo`, `npm`, `npx`, `gh issue create`/`comment`/`edit`,
SQL d'écriture. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`, `gh api` en lecture, `pdftotext` vers
`target/gate-logs/`.
