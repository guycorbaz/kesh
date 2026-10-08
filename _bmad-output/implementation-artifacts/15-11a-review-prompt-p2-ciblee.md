# Prompt — revue de code P2 ciblée, Story 15-11a

*Versionné le 2026-10-08. Passe ciblée (CLAUDE.md § « La passe ciblée ») : une seule lentille (Haiku), contexte
frais, en lecture seule, braquée sur la seule remédiation de la revue P1.*

Worktree `/home/gcorbaz/devel/kesh-15-11a`, branche `story/15-11a-compose-transmet-la-configuration`. **Objet** :
`git show 47a1a656` (un seul commit, rebasé sur `ec675288`). Rapports remédiés :
`target/gate-logs/15-11a-review-p1-{B,E,A}.md`. Choix C-15-11a-6.

## Lentille unique — chasseur de régressions de la remédiation

1. **Lectures « vide = défaut »** (`crates/kesh-api/src/config.rs`) : `KESH_ADMIN_BACKUP_DIR` et `KESH_LANG` par
   `opt_trimmed_env` ; les cinq numériques (`KESH_PASSWORD_MIN_LENGTH`, `KESH_BANK_IMPORT_MAX_MB`,
   `KESH_ADMIN_EXPORT_INMEM_MB`, `KESH_ADMIN_IMPORT_MAX_MB`, `KESH_SMTP_PORT`) avec un bras
   `Ok(val) if val.trim().is_empty()`. Pour chacune : le défaut appliqué est-il exactement celui d'avant (relis le
   diff) ? une valeur non vide invalide garde-t-elle son avertissement ou son refus d'avant ? `KESH_LANG` trimé
   (« de » au lieu de « de ») change-t-il un comportement ailleurs (`grep -rn KESH_LANG crates/`) ?
2. **Tests** `from_env_empty_or_blank_vars_take_code_default_silently` et
   `from_env_non_empty_invalid_values_still_warn` : la capture `tracing` locale est-elle non muette (assertion
   témoin) ? Les tests manipulent des variables d'environnement globales : sont-ils sérialisés comme les autres tests
   de `config.rs` (`reset_env`, verrou, `serial`) — sinon, faux rouge ou faux vert possible en parallèle ?
3. **Doc-comments** rattachés (`parse_strict_bool`, `is_loopback_host`, `TEMPLATE_PLACEHOLDERS`,
   `is_template_placeholder`).
4. **Textes** : commentaire de tête des compose, manuel (`:664`, `:757`, `:1692`, et le PDF aplati : `pdftotext
   -nopgbrk docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' '`, ligatures normalisées), `.env.example:170`,
   CHANGELOG (#550) : disent-ils exactement « vide = défaut » et rien de plus ?

## Ce que tu rends

Findings numérotés (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (sortie de `grep -nF` ; code relu cité),
correction proposée. ⛔ **Liste des axes exercés ET non exercés** — un « 0 » sans elle ne compte pas. Rapport
complet dans `target/gate-logs/15-11a-review-p2-ciblee.md` ; dernier message : le chemin, le bilan, une ligne par
MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `target/gate-logs/`. Aucune commande qui écrit dans le dépôt ou dans une base, ni qui
compile ou exécute des tests : `scripts/*` (dont `scripts/prepare-release.sh`), `make`, `latexmk`,
`git commit`/`add`/`checkout`/`stash`, `sqlx`, `cargo`, `npm`, `npx`, `gh issue create`/`comment`/`edit`, SQL
d'écriture. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`, `gh api` en lecture, `pdftotext` vers
`target/gate-logs/`.
