# Prompt — validation P3, Story 25-6-b (le PDF d'une facture est figé)

*Versionné le 2026-10-05. Une passe (Sonnet), contexte frais. Trend : P1 4H/12M (Sonnet ×3) → P2 1H/7M/8L (Opus) ; pas de
découpage (Guy). La remédiation de P2 change une règle de concurrence (garde `version`) et ajoute des sites de manuel :
fiche entière, attention particulière au diff.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-6-b-pdf-facture-archive`. **Fiche :**
`_bmad-output/implementation-artifacts/25-6-b-pdf-facture-archive.md`. Remédiation de P2 : `git diff ac125b29 HEAD`.
Règles : `CLAUDE.md`. Les huit arbitrages sont rendus par Guy.

## Ce que tu vérifies

1. **La garde `version` (AC 2)** : `render` peut-il exposer la version **lue** sans changer son comportement pour ses
   autres appelants (`grep -rn "invoice_pdf_service::render\|render_document" crates`) ? La règle « une seule nouvelle
   tentative, puis 409 `INVOICE_CHANGED` » est-elle cohérente avec les trois cas « zéro ligne » ? Le gel ne touche pas
   `version` : la garde `version = ?` reste-t-elle juste si **un autre** chemin incrémente `version` sans changer le
   contenu rendu (envoi par e-mail `mark_emailed`, pause des rappels, règlement — `grep -n "version = version + 1"
   crates/kesh-db/src/repositories/invoices.rs`) ? Faut-il alors une nouvelle tentative fréquente, voire un 409 en usage
   normal ?
2. **Les sites de manuel ajoutés** (AC 8) : chacun existe-t-il et dit-il ce que la fiche affirme ? `sed -n` sur
   `docs/manual/fr/admin-manual.tex` (`:1353`, `:1376-1381`, `:1444`, `:1480`, `:1583`, `:1648-1654`, `:1774`) et
   `user-manual.tex` (`:789`, `:797`, `:815`, `:846`).
3. **Les gardes structurelles** (AC 3-bis) : le compte `lib.rs:186` (25), `admin_pat_denied_e2e.rs:46`, `:757`,
   `audit_route_registry.rs` (`LIB_ROUTES`, `extract_counted` `:186`), `audit_label_registry.rs` — existent-ils tels que
   la fiche les décrit ?
4. **Cohérence interne** : arbitrages, AC, tâches, limites, *Modules*, Change Log — y a-t-il une phrase qui contredit une
   autre depuis la remédiation (par exemple « jamais régénérer » contre refiger ; codes HTTP de la même situation écrits
   deux fois différemment ; « six » contre « quatre » sites) ? `grep -nE "409|410|500" <fiche>`.

## Ce que tu rends

Findings avec sévérité (CRITICAL/HIGH/MEDIUM/LOW), endroit, **preuve** (commande ET sortie, `grep -nF` pour toute
présence ou absence), correction proposée. ⛔ La liste des axes exercés ET non exercés.

## Interdits

⛔ N'écris aucun fichier du dépôt hors `target/gate-logs/` ; aucune commande qui écrit dans le dépôt ou dans une base :
`scripts/*` (dont `scripts/prepare-release.sh`), `make`, `latexmk`, `git commit`/`add`/`checkout`, `sqlx`,
`cargo test`/`nextest`, `npm run`, `npx`, `gh issue create`/`comment`/`edit`. Autorisés : lecture, `grep`, `sed`,
`git log`/`show`/`diff`, `gh issue view`.
