# Prompt — validation P4 ciblée, Story 25-6-b

*Versionné le 2026-10-05. **Une lentille** (Haiku), contexte frais — passe ciblée sur la remédiation de P3. Trend :
P1 4H/12M → P2 1H/7M → P3 3M/4L.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-6-b-pdf-facture-archive`. **Diff à relire, un seul commit, aplati :
`git show 54337553`** (la fiche `_bmad-output/implementation-artifacts/25-6-b-pdf-facture-archive.md`). La fiche décrit du
travail **à faire** : un finding qui reproche au code de ne pas encore le porter sera rejeté.

## Lentille — regression hunter

1. **Les trois totaux du registre** : `sed -n 430,475p crates/kesh-api/tests/audit_route_registry.rs` — les valeurs 109,
   91, 112 y sont-elles, et la fiche les porte-t-elle à 110, 92, 113 ?
2. **Les six variantes d'erreur** : chacune est-elle employée de façon cohérente dans les AC (`grep -nE
   "INVOICE_PDF_GONE|INVOICE_CHANGED|INVOICE_PDF_PRESENT|INVOICE_PDF_NOT_FROZEN|INVOICE_PDF_INTEGRITY|InvoiceCancelled" <fiche>`)
   — même statut HTTP partout où la même situation est décrite ? `InvoicePdfIntegrity` est dite « 409 au refigeage, 500 à
   la lecture » : l'AC 3 dit-elle bien 500 pour la lecture ?
3. **`render_document` porte la version** : `sed -n 195,220p crates/kesh-api/src/routes/invoice_pdf_service.rs` —
   charge-t-il bien la facture ? Ses appelants « rappel » (`grep -n "render_document" crates/kesh-api/src/routes/invoice_email.rs`)
   existent-ils aux lignes citées ?
4. **Résidus** : la fiche dit-elle encore « six sites » pour H1 de P2 ? `grep -n "Six sites\|six sites" <fiche>`.

## Ce que tu rends

Findings avec sévérité, endroit, **preuve obligatoire : la commande ET sa sortie** — un finding sans sortie sera rejeté,
un « 0 finding » sans les sorties des quatre vérifications aussi. ⛔ La liste des axes exercés ET non exercés.

## Interdits

⛔ N'écris aucun fichier du dépôt ; aucune commande qui écrit dans le dépôt ou dans une base (`scripts/*` dont
`scripts/prepare-release.sh`, `make`, `latexmk`, `git commit`/`add`/`checkout`, `sqlx`, `cargo test`/`nextest`,
`npm run`, `npx`, `gh issue create`/`comment`/`edit`). Autorisés : lecture, `grep`, `sed`, `git log`/`show`/`diff`.
